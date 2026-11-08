#!/usr/bin/env python3
"""Local human-judgment workbench. Python standard library; Rust builds questions."""
import argparse
from contextlib import closing
from datetime import datetime, timezone
import hashlib
from http.server import BaseHTTPRequestHandler, ThreadingHTTPServer
import json
from pathlib import Path
import random
import sqlite3
import subprocess
import sys
import urllib.parse
import webbrowser

ROOT = Path(__file__).resolve().parents[1]
sys.path.insert(0, str(ROOT))
from scripts.export_human_games import sanitize

ASSETS = ROOT / 'scripts' / 'judgment_ui'
DEFAULT_STORE = ROOT / 'data' / 'judgments' / 'judgments.db'
SEATS = ['north', 'east', 'south', 'west']
KINDS = ['pass', 'charge', 'play']
DIRECTIONS = ['left', 'right', 'across', 'keeper']
DECK = [r+s for s in 'CDHS' for r in '23456789TJQKA']
INTENTS = ['avoid_points', 'shoot_moon', 'stop_moon', 'win_jack', 'manage_ten',
           'create_void', 'keep_control', 'shed_low_cards', 'give_points', 'other']


class Conflict(ValueError):
    pass


def connect(path):
    conn = sqlite3.connect(path, timeout=10)
    conn.row_factory = sqlite3.Row
    conn.execute('PRAGMA foreign_keys=ON')
    return conn


def initialize(path):
    path = Path(path)
    path.parent.mkdir(parents=True, exist_ok=True)
    with closing(connect(path)) as conn, conn:
        conn.execute('PRAGMA journal_mode=WAL')
        conn.executescript('''
            CREATE TABLE IF NOT EXISTS cases (
              id TEXT PRIMARY KEY, group_id TEXT NOT NULL, kind TEXT NOT NULL,
              direction TEXT NOT NULL, payload TEXT NOT NULL, queue_order INTEGER NOT NULL);
            CREATE TABLE IF NOT EXISTS reviews (
              id INTEGER PRIMARY KEY, case_id TEXT NOT NULL REFERENCES cases(id),
              reviewer TEXT NOT NULL, revision INTEGER NOT NULL, status TEXT NOT NULL,
              payload TEXT NOT NULL, created_at TEXT NOT NULL,
              UNIQUE(case_id, reviewer, revision));
            CREATE INDEX IF NOT EXISTS review_latest ON reviews(reviewer, case_id, revision DESC);
        ''')


def reviewer_name(value):
    if not isinstance(value, str) or not 1 <= len(value.strip()) <= 80:
        raise ValueError('Enter a reviewer name of 1–80 characters.')
    return value.strip()


def latest(conn, case_id, reviewer):
    return conn.execute('SELECT * FROM reviews WHERE case_id=? AND reviewer=? '
                        'ORDER BY revision DESC LIMIT 1', (case_id, reviewer)).fetchone()


def progress(conn, reviewer):
    rows = conn.execute('''SELECT c.kind, r.status, count(*) AS n FROM cases c
      LEFT JOIN reviews r ON r.id=(SELECT id FROM reviews WHERE case_id=c.id AND reviewer=?
                                   ORDER BY revision DESC LIMIT 1)
      GROUP BY c.kind, r.status''', (reviewer,)).fetchall()
    result = {k: {'pending': 0, 'answered': 0, 'skipped': 0} for k in KINDS}
    for row in rows:
        result[row['kind']][row['status'] or 'pending'] = row['n']
    return result


def public_state(path, reviewer, kind='all', direction='all', case_id=None):
    reviewer = reviewer_name(reviewer)
    if kind not in KINDS + ['all'] or direction not in DIRECTIONS + ['all']:
        raise ValueError('Unknown decision or pass direction.')
    with closing(connect(path)) as conn:
        stats = progress(conn, reviewer)
        if case_id:
            row = conn.execute('SELECT * FROM cases WHERE id=?', (case_id,)).fetchone()
            if row is None:
                raise ValueError('Question not found.')
        else:
            row = conn.execute('''SELECT c.* FROM cases c WHERE (?='all' OR kind=?)
                AND (?='all' OR direction=?) AND NOT EXISTS
                (SELECT 1 FROM reviews r WHERE r.case_id=c.id AND r.reviewer=?)
                ORDER BY queue_order, id LIMIT 1''', (kind, kind, direction, direction, reviewer)).fetchone()
        case = None
        if row:
            stored = json.loads(row['payload'])
            review = latest(conn, row['id'], reviewer)
            # Explicit allowlist: never send policy choices or internal context.
            case = {'id': row['id'], 'view': stored['view'],
                    'revision': review['revision'] if review else 0,
                    'annotation': json.loads(review['payload']) if review else None}
        recent = [dict(r) for r in conn.execute('''SELECT r.case_id, c.kind, c.direction,
            r.status, r.created_at FROM reviews r JOIN cases c ON c.id=r.case_id
            WHERE r.reviewer=? AND r.id=(SELECT id FROM reviews rr WHERE rr.case_id=r.case_id
            AND rr.reviewer=r.reviewer ORDER BY revision DESC LIMIT 1)
            ORDER BY r.id DESC LIMIT 20''', (reviewer,))]
    return {'case': case, 'progress': stats, 'recent': recent, 'intents': INTENTS}


def canonical_action(cards, view):
    if not isinstance(cards, list) or any(not isinstance(c, str) for c in cards):
        raise ValueError('Choose cards from the displayed hand.')
    if len(cards) != len(set(cards)) or any(c not in view['legal_cards'] for c in cards):
        raise ValueError('That choice contains duplicate or illegal cards.')
    if not view['min_cards'] <= len(cards) <= view['max_cards']:
        raise ValueError(f"Choose {view['min_cards']}–{view['max_cards']} cards.")
    return sorted(cards, key=DECK.index)


def save_review(path, body):
    if not isinstance(body, dict):
        raise ValueError('Expected a judgment object.')
    reviewer = reviewer_name(body.get('reviewer'))
    expected = body.get('expected_revision')
    if type(expected) is not int or expected < 0:
        raise ValueError('Missing question revision. Reload the question.')
    status = body.get('status')
    if status not in ['answered', 'skipped']:
        raise ValueError('Unknown judgment status.')
    with closing(connect(path)) as conn, conn:
        conn.execute('BEGIN IMMEDIATE')
        row = conn.execute('SELECT payload FROM cases WHERE id=?', (body.get('case_id'),)).fetchone()
        if row is None:
            raise ValueError('Question not found.')
        previous = latest(conn, body['case_id'], reviewer)
        if expected != (previous['revision'] if previous else 0):
            raise Conflict('This judgment changed in another tab. Reload before editing it.')
        case = json.loads(row['payload'])
        action = canonical_action(body.get('action'), case['view']) if status == 'answered' else None
        alternatives = body.get('alternatives', []) if status == 'answered' else []
        if not isinstance(alternatives, list) or len(alternatives) > 20:
            raise ValueError('Supply at most 20 alternative choices.')
        alternatives = [canonical_action(a, case['view']) for a in alternatives]
        if len({tuple(a) for a in alternatives}) != len(alternatives) or action in alternatives:
            raise ValueError('Alternative choices must be distinct from each other and the preferred choice.')
        confidence = body.get('confidence') or None
        if confidence not in [None, 'high', 'medium', 'low']:
            raise ValueError('Unknown confidence level.')
        intents = body.get('intents', [])
        if not isinstance(intents, list) or any(i not in INTENTS for i in intents):
            raise ValueError('Unknown strategic intent.')
        runner = body.get('suspected_runner', 'unknown')
        if runner not in SEATS + ['unknown', 'none']:
            raise ValueError('Unknown suspected runner.')
        note = body.get('note', '')
        if not isinstance(note, str) or len(note) > 4000:
            raise ValueError('Notes must be at most 4,000 characters.')
        recognized = body.get('recognized', False)
        if type(recognized) is not bool:
            raise ValueError('Recognition must be true or false.')
        seconds = body.get('viewing_seconds', 0)
        if type(seconds) not in [int, float] or not 0 <= seconds <= 86400:
            raise ValueError('Invalid viewing duration.')
        label = {'status': status, 'action': action, 'alternatives': alternatives,
                 'confidence': confidence, 'intents': sorted(set(intents)),
                 'suspected_runner': runner, 'note': note.strip(), 'recognized': recognized,
                 'viewing_seconds': round(seconds, 2), 'label_semantics': 'expert_recommendation'}
        now = datetime.now(timezone.utc).isoformat()
        conn.execute('INSERT INTO reviews(case_id,reviewer,revision,status,payload,created_at) '
                     'VALUES(?,?,?,?,?,?)', (body['case_id'], reviewer, expected+1, status,
                                            json.dumps(label), now))
    return {'saved': True, 'revision': expected+1}


def import_cases(path, cases):
    added = 0
    with closing(connect(path)) as conn, conn:
        order = conn.execute('SELECT coalesce(max(queue_order),-1)+1 FROM cases').fetchone()[0]
        for case in cases:
            if case.get('schema') != 'turbo-hearts-judgment-v1':
                raise ValueError('Unsupported question schema.')
            view = case['view']
            if view['kind'] not in KINDS or view['direction'] not in DIRECTIONS:
                raise ValueError('Invalid question kind/direction.')
            if not set(view['legal_cards']) <= set(view['hand']) <= set(DECK):
                raise ValueError('Invalid question cards.')
            previous = conn.execute('SELECT payload FROM cases WHERE id=?', (case['id'],)).fetchone()
            if previous:
                if json.loads(previous[0])['view'] != view:
                    raise ValueError('An existing question has changed. Use a new version/ID.')
                continue
            conn.execute('INSERT INTO cases VALUES(?,?,?,?,?,?)', (case['id'], case['group_id'],
                         view['kind'], view['direction'], json.dumps(case), order+added))
            added += 1
    return added


def select_cases(cases, count, seed, existing=()):
    """Balanced tasks/directions, alternating disagreement enrichment and random controls."""
    rng = random.Random(seed)
    buckets = {}
    for case in cases:
        if case['id'] in existing:
            continue
        view = case['view']
        key = (view['kind'], view['direction'])
        buckets.setdefault(key, []).append(case)
    for pool in buckets.values():
        rng.shuffle(pool)
    chosen, per_group = [], {}
    keys = list(buckets)
    rng.shuffle(keys)
    round_number = 0
    while len(chosen) < count and any(buckets.values()):
        for key in keys:
            pool = buckets[key]
            if not pool or len(chosen) == count:
                continue
            # Prefer less-repeated games, and enrich half the sweeps with disagreements.
            enrich = round_number % 2 == 0
            index = min(range(len(pool)), key=lambda i: (
                per_group.get(pool[i]['group_id'], 0),
                not pool[i]['selection']['disagreement'] if enrich else False))
            case = pool.pop(index)
            case['selection']['queue_stratum'] = 'disagreement' if enrich else 'random_control'
            case['selection']['queue_seed'] = seed
            chosen.append(case)
            per_group[case['group_id']] = per_group.get(case['group_id'], 0)+1
        round_number += 1
    return chosen


def prepare(args):
    if args.count < 1 or args.max_games < 1:
        raise ValueError('Counts must be positive.')
    paths = args.database or [p for p in [ROOT/'turbo-hearts.db', ROOT/'th2.db'] if p.exists()]
    if not paths:
        raise ValueError('Provide --database PATH to a game database.')
    folder = Path(args.store).resolve().parent
    folder.mkdir(parents=True, exist_ok=True)
    source = folder/'source-games.jsonl'
    seen = set()
    rng = random.Random(args.seed)
    with source.open('w') as out:
        for path in paths:
            with closing(sqlite3.connect(Path(path).resolve().as_uri()+'?mode=ro', uri=True)) as db:
                games = db.execute('''SELECT game_id FROM game g WHERE completed_time IS NOT NULL
                    AND EXISTS(SELECT 1 FROM game_player p WHERE p.game_id=g.game_id AND strategy IS NULL)
                    ORDER BY completed_time, game_id''').fetchall()
                candidates = []
                for i, (game_id,) in enumerate(games):
                    split = 0 if i < int(len(games)*.7) else 1 if i < int(len(games)*.85) else 2
                    if split == 0 or args.include_evaluation:
                        candidates.append((game_id, split))
                rng.shuffle(candidates)
                for game_id, split in candidates[:args.max_games]:
                    group = hashlib.sha256(('judgment-v1:'+game_id).encode()).hexdigest()[:24]
                    if group in seen:
                        continue
                    seen.add(group)
                    events = [json.loads(r[0]) for r in db.execute(
                        'SELECT event FROM event WHERE game_id=? ORDER BY event_id', (game_id,))]
                    events = sanitize(events)
                    humans = [events[0][seat]['type']=='human' for seat in SEATS]
                    out.write(json.dumps({'group_id':group,'split':split,'humans':humans,'events':events})+'\n')
    if not seen:
        raise ValueError('No eligible complete human games; training games are selected by default.')
    if not args.no_build:
        subprocess.run(['cargo','build','--locked','--release','-p','turbo-hearts-bot',
                        '--example','judgment_cases'], cwd=ROOT, check=True)
    generated = folder/'candidate-questions.jsonl'
    subprocess.run([str(ROOT/'target/release/examples/judgment_cases'), str(source), str(generated),
                    str(args.per_kind), str(args.seed)], cwd=ROOT, check=True)
    with closing(connect(args.store)) as conn:
        existing = {r[0] for r in conn.execute('SELECT id FROM cases')}
    with generated.open() as stream:
        selected = select_cases([json.loads(line) for line in stream], args.count, args.seed, existing)
    added = import_cases(args.store, selected)
    print(json.dumps({'added':added,'source_games':len(seen),'store':str(args.store),
                      'tasks':{k:sum(c['view']['kind']==k for c in selected) for k in KINDS}}))


def export_labels(args):
    with closing(connect(args.store)) as conn, args.output.open('w') as out:
        rows = conn.execute('''SELECT c.payload AS question, r.* FROM reviews r JOIN cases c ON c.id=r.case_id
            WHERE r.id=(SELECT id FROM reviews rr WHERE rr.case_id=r.case_id AND rr.reviewer=r.reviewer
                        ORDER BY revision DESC LIMIT 1) ORDER BY r.id''')
        count = 0
        for row in rows:
            label = json.loads(row['payload'])
            if args.reviewer and row['reviewer'] != args.reviewer:
                continue
            if row['status'] != 'answered' or (label['recognized'] and not args.include_recognized):
                continue
            out.write(json.dumps({'schema':'turbo-hearts-judgment-label-v1',
                'question':json.loads(row['question']), 'reviewer':row['reviewer'],
                'revision':row['revision'],'created_at':row['created_at'],'judgment':label})+'\n')
            count += 1
    print(json.dumps({'exported':count,'output':str(args.output)}))


def make_server(path, port):
    class Handler(BaseHTTPRequestHandler):
        def log_message(self, *_):
            pass

        def respond(self, value, status=200):
            data = json.dumps(value).encode()
            self.send_response(status)
            self.send_header('Content-Type', 'application/json; charset=utf-8')
            self.send_header('Cache-Control', 'no-store')
            self.send_header('X-Content-Type-Options', 'nosniff')
            self.send_header('Content-Length', str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def valid_host(self):
            return self.headers.get('Host') in [f'127.0.0.1:{self.server.server_port}',
                                               f'localhost:{self.server.server_port}']

        def do_GET(self):
            if not self.valid_host():
                return self.respond({'error':'Invalid host'}, 403)
            parsed = urllib.parse.urlparse(self.path)
            if parsed.path == '/api/state':
                params = urllib.parse.parse_qs(parsed.query)
                get = lambda k, default=None: params.get(k, [default])[0]
                try:
                    return self.respond(public_state(path, get('reviewer','local'),
                        get('kind','all'),get('direction','all'),get('case')))
                except ValueError as e:
                    return self.respond({'error':str(e)}, 400)
            asset = {'/':'index.html','/app.js':'app.js','/style.css':'style.css'}.get(parsed.path)
            if asset is None:
                return self.respond({'error':'Not found'}, 404)
            data = (ASSETS/asset).read_bytes()
            self.send_response(200)
            self.send_header('Content-Type', {'index.html':'text/html; charset=utf-8',
                'app.js':'text/javascript; charset=utf-8','style.css':'text/css; charset=utf-8'}[asset])
            self.send_header('Cache-Control','no-store')
            self.send_header('Content-Security-Policy', "default-src 'self'; style-src 'self'; script-src 'self'; frame-ancestors 'none'")
            self.send_header('Content-Length',str(len(data)))
            self.end_headers()
            self.wfile.write(data)

        def do_POST(self):
            origin = self.headers.get('Origin')
            if not self.valid_host() or (origin and origin not in [
                f'http://127.0.0.1:{self.server.server_port}',f'http://localhost:{self.server.server_port}']):
                return self.respond({'error':'Invalid origin'}, 403)
            if self.path != '/api/review':
                return self.respond({'error':'Not found'}, 404)
            try:
                size = int(self.headers.get('Content-Length','0'))
                if not 0 < size <= 65536 or self.headers.get('Content-Type','').split(';')[0] != 'application/json':
                    raise ValueError('Expected a small JSON request.')
                result = save_review(path, json.loads(self.rfile.read(size)))
                return self.respond(result)
            except Conflict as e:
                return self.respond({'error':str(e)}, 409)
            except (ValueError, TypeError) as e:
                return self.respond({'error':str(e)}, 400)
    return ThreadingHTTPServer(('127.0.0.1', port), Handler)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--store', type=Path, default=DEFAULT_STORE)
    subs = parser.add_subparsers(dest='command', required=True)
    p = subs.add_parser('prepare', help='Build new questions from local game databases, read-only.')
    p.add_argument('--database', type=Path, action='append')
    p.add_argument('--count', type=int, default=240)
    p.add_argument('--max-games', type=int, default=80, help='Maximum source games per database.')
    p.add_argument('--per-kind', type=int, default=4, help='Candidate positions per decision type per game.')
    p.add_argument('--seed', type=int, default=42)
    p.add_argument('--include-evaluation', action='store_true', help='Also review validation/test games; keep their split on export.')
    p.add_argument('--no-build', action='store_true')
    p = subs.add_parser('serve', help='Open the local review interface.')
    p.add_argument('--port', type=int, default=8765)
    p.add_argument('--open', action='store_true', help='Open your default browser.')
    p = subs.add_parser('export', help='Export latest answered labels; skips/recognized hands excluded by default.')
    p.add_argument('output', type=Path)
    p.add_argument('--reviewer')
    p.add_argument('--include-recognized', action='store_true')
    p = subs.add_parser('stats', help='Show a reviewer’s progress.')
    p.add_argument('--reviewer', default='local')
    args = parser.parse_args()
    initialize(args.store)
    try:
        if args.command == 'prepare':
            prepare(args)
        elif args.command == 'export':
            args.output.parent.mkdir(parents=True, exist_ok=True)
            if args.output.resolve() == args.store.resolve():
                raise ValueError('The export must not overwrite the judgment database.')
            export_labels(args)
        elif args.command == 'stats':
            with closing(connect(args.store)) as conn:
                print(json.dumps(progress(conn, reviewer_name(args.reviewer)), indent=2))
        else:
            server = make_server(args.store, args.port)
            url = f'http://127.0.0.1:{server.server_port}'
            print(f'Human judgment workbench: {url}\nProgress: {args.store}\nCtrl-C to stop.', flush=True)
            if args.open:
                webbrowser.open(url)
            try:
                server.serve_forever()
            except KeyboardInterrupt:
                pass
            finally:
                server.server_close()
    except (ValueError, OSError, sqlite3.Error, subprocess.CalledProcessError) as e:
        parser.exit(1, f'Error: {e}\n')


if __name__ == '__main__':
    main()
