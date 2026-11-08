#!/usr/bin/env python3
"""Package decision-lab questions and export responses from the real server DB, read-only."""
import argparse
from contextlib import closing
import hashlib
import json
from pathlib import Path
import sqlite3

ROOT = Path(__file__).resolve().parents[1]


def readonly(path):
    conn = sqlite3.connect(Path(path).resolve().as_uri() + '?mode=ro', uri=True)
    conn.row_factory = sqlite3.Row
    return conn


def output_file(source, output):
    output = Path(output)
    if output.suffix != '.jsonl' or output.resolve() == Path(source).resolve():
        raise ValueError('Choose a separate .jsonl output file.')
    output.parent.mkdir(parents=True, exist_ok=True)
    return output.open('w')


def pack(store, output):
    with closing(readonly(store)) as db, output_file(store, output) as out:
        count = 0
        for row in db.execute('SELECT payload FROM cases ORDER BY queue_order,id'):
            case = json.loads(row[0])
            if case['schema'] != 'turbo-hearts-judgment-v1':
                raise ValueError('Unsupported question schema.')
            out.write(json.dumps(case, separators=(',', ':')) + '\n')
            count += 1
    return {'questions': count, 'output': str(output)}


def export(database, output, include_recognized=False, raw_user_ids=False):
    with closing(readonly(database)) as db, output_file(database, output) as out:
        count = 0
        for row in db.execute('''SELECT c.payload AS question,r.* FROM judgment_response r
            JOIN judgment_case c ON c.id=r.case_id WHERE revision=(SELECT max(rr.revision)
            FROM judgment_response rr WHERE rr.case_id=r.case_id AND rr.user_id=r.user_id)
            ORDER BY r.id'''):
            label = json.loads(row['payload'])
            if row['status'] != 'answered' or (label['recognized'] and not include_recognized):
                continue
            reviewer = row['user_id'] if raw_user_ids else hashlib.sha256(
                ('judgment-reviewer-v1:' + row['user_id']).encode()).hexdigest()[:24]
            out.write(json.dumps({'schema': 'turbo-hearts-judgment-label-v1',
                'question': json.loads(row['question']), 'reviewer': reviewer,
                'revision': row['revision'], 'created_at': row['created_at'],
                'judgment': label}, separators=(',', ':')) + '\n')
            count += 1
    return {'exported': count, 'output': str(output)}


def stats(database):
    with closing(readonly(database)) as db:
        totals = dict(db.execute('''SELECT status,count(*) FROM judgment_response r
            WHERE revision=(SELECT max(rr.revision) FROM judgment_response rr
                WHERE rr.user_id=r.user_id AND rr.case_id=r.case_id) GROUP BY status'''))
        return {'questions': db.execute('SELECT count(*) FROM judgment_case').fetchone()[0],
            'reviewers': db.execute('SELECT count(DISTINCT user_id) FROM judgment_response').fetchone()[0],
            'revisions': db.execute('SELECT count(*) FROM judgment_response').fetchone()[0],
            'latest': totals}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    subs = parser.add_subparsers(dest='command', required=True)
    p = subs.add_parser('pack', help='Package prepared questions for judgment_cases_path; keep outside public assets.')
    p.add_argument('output', type=Path)
    p.add_argument('--store', type=Path, default=ROOT/'data/judgments/judgments.db')
    p = subs.add_parser('export', help='Export latest answered labels with complete decision context.')
    p.add_argument('output', type=Path)
    p.add_argument('--database', type=Path, required=True)
    p.add_argument('--include-recognized', action='store_true')
    p.add_argument('--raw-user-ids', action='store_true', help='Allow offline joins to player game performance.')
    p = subs.add_parser('stats')
    p.add_argument('--database', type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.command == 'pack':
            result = pack(args.store, args.output)
        elif args.command == 'export':
            result = export(args.database, args.output, args.include_recognized, args.raw_user_ids)
        else:
            result = stats(args.database)
        print(json.dumps(result, indent=2))
    except (ValueError, OSError, sqlite3.Error) as e:
        parser.exit(1, f'Error: {e}\n')


if __name__ == '__main__':
    main()
