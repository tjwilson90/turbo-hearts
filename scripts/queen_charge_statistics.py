#!/usr/bin/env python3
"""Describe human Q-spade holdings at first play, before/after voluntary charges.

Input is the sanitized JSONL from export_human_games.py. These are descriptive
counts, not held-out evidence for a learned hidden-hand model.
"""
import argparse
import json
from collections import Counter
from pathlib import Path

SEATS = ("north", "east", "south", "west")


def statistics(games):
    held, charged = Counter(), Counter()
    for game in games:
        events = game["events"]
        if events[0]["rules"] != "classic":
            continue
        hands, charges, waiting = {}, set(), False
        for event in events:
            kind = event["type"]
            if kind == "deal":
                hands = {seat: set(event[seat]) for seat in SEATS}
                charges, waiting = set(), True
            elif kind == "send_pass":
                hands[event["from"]].difference_update(event["cards"])
            elif kind == "recv_pass":
                hands[event["to"]].update(event["cards"])
            elif kind == "charge":
                charges.update(event["cards"])
            elif kind == "play" and waiting:
                waiting = False
                for seat, human in zip(SEATS, game["humans"]):
                    hand = hands[seat]
                    if not human or "QS" not in hand:
                        continue
                    spades = sum(card.endswith("S") for card in hand)
                    features = {
                        "hands": 1,
                        "spades_at_least_5": int(spades >= 5),
                        "spades_at_least_6": int(spades >= 6),
                        "ace_spades": int("AS" in hand),
                        "nine_spades": int("9S" in hand),
                        "ace_and_nine_spades": int({"AS", "9S"} <= hand),
                    }
                    held.update(features)
                    if "QS" in charges:
                        charged.update(features)
    return {"held": dict(held), "charged": dict(charged)}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("games", type=Path)
    args = parser.parse_args()
    with args.games.open() as source:
        print(json.dumps(statistics(json.loads(line) for line in source), indent=2))


if __name__ == "__main__":
    main()
