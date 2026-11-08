#!/usr/bin/env python3
"""Export complete human games read-only, removing identities, chats and seeds.

Whole games are split chronologically: 70% training, 15% validation, 15% test.
Generated event data stays in ignored data/; never publish the source database.
"""
import argparse
import json
import sqlite3
from pathlib import Path


def sanitize(events):
    result = []
    for event in events:
        if event["type"] in {"chat", "join_game", "leave_game"}:
            continue
        if event["type"] == "sit":
            # Older records also contain created_by and created_time. Retain
            # only game semantics, rather than copying incidental metadata.
            event = {key: event[key] for key in
                     ("type", "north", "east", "south", "west", "rules") if key in event}
            event["seed"] = {"type": "chosen", "value": "historical-replay"}
            for index, seat in enumerate(("north", "east", "south", "west")):
                player = event[seat]
                event[seat] = {"type": player["type"],
                               "user_id": f"00000000-0000-0000-0000-{index+1:012d}"}
                if player["type"] == "bot":
                    event[seat]["strategy"] = player["strategy"]
        result.append(event)
    return result


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("database", type=Path)
    parser.add_argument("output", type=Path)
    args = parser.parse_args()
    db = sqlite3.connect(args.database.resolve().as_uri() + "?mode=ro", uri=True)
    games = db.execute("""SELECT game_id FROM game g WHERE completed_time IS NOT NULL
        AND EXISTS (SELECT 1 FROM game_player p WHERE p.game_id=g.game_id AND strategy IS NULL)
        ORDER BY completed_time, game_id""").fetchall()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    counts = [0, 0, 0]
    with args.output.open("w") as out:
        for index, (game_id,) in enumerate(games):
            events = [json.loads(row[0]) for row in db.execute(
                "SELECT event FROM event WHERE game_id=? ORDER BY event_id", (game_id,))]
            assert events[0]["type"] == "sit"
            humans = [events[0][s]["type"] == "human" for s in ("north", "east", "south", "west")]
            split = 0 if index < int(len(games)*.7) else 1 if index < int(len(games)*.85) else 2
            counts[split] += 1
            out.write(json.dumps({"id": index, "split": split, "humans": humans,
                                  "events": sanitize(events)}, separators=(",", ":")) + "\n")
    print(json.dumps({"games": len(games), "split_games": counts}))


if __name__ == "__main__":
    main()
