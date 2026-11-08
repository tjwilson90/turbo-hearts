#!/usr/bin/env python3
"""Read-only anonymous corpus with stable player keys for training-only ratings.

Raw identifiers are mapped locally to integers, never written. Recent games
form a separate diagnostic split. Run corpus_scores to recover engine payoffs.
"""
import argparse
import json
import sqlite3
from pathlib import Path
from export_human_games import sanitize


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("database", type=Path)
    parser.add_argument("output", type=Path)
    parser.add_argument("--recent", type=Path)
    args = parser.parse_args()
    players, seen = {}, set()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    counts = [0] * 4
    with args.output.open("w") as out:
        for recent, database in enumerate([args.database] + ([args.recent] if args.recent else [])):
            db = sqlite3.connect(database.resolve().as_uri()+"?mode=ro", uri=True)
            games = db.execute('''SELECT game_id FROM game g WHERE completed_time IS NOT NULL
                AND EXISTS(SELECT 1 FROM game_player p WHERE p.game_id=g.game_id AND strategy IS NULL)
                ORDER BY completed_time,game_id''').fetchall()
            for index, (game_id,) in enumerate(games):
                if game_id in seen:
                    continue
                seen.add(game_id)
                events = [json.loads(r[0]) for r in db.execute(
                    "SELECT event FROM event WHERE game_id=? ORDER BY event_id", (game_id,))]
                sit = events[0]
                human, keys = [], []
                for seat in ["north", "east", "south", "west"]:
                    p = sit[seat]
                    human.append(p["type"] == "human")
                    key = p["user_id"] if human[-1] else "bot:"+p["strategy"]
                    if key not in players:
                        players[key] = len(players)
                    keys.append(players[key])
                split = 3 if recent else 0 if index < int(.7*len(games)) else 1 if index < int(.85*len(games)) else 2
                row = {"id": len(seen)-1, "split": split, "humans": human,
                       "players": keys, "events": sanitize(events)}
                out.write(json.dumps(row, separators=(",", ":"))+"\n")
                counts[split] += 1
    print(json.dumps({"games_by_split": counts, "anonymous_player_or_bot_keys": len(players)}))


if __name__ == "__main__":
    main()
