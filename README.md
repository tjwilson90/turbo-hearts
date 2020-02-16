# Turbo Hearts

![](https://github.com/tjwilson90/turbo-hearts/workflows/Rust/badge.svg)

## Getting Started

1) Install rust - https://www.rust-lang.org/tools/install
2) Run the server - `cargo run`
3) Run the client - http://localhost:7380/lobby

## Endpoints

All endpoints require the caller to pass a `name` cookie identifying themselves. There's no
authentication; please don't cheat.

### `GET /lobby`

Returns an html page for the game lobby displaying the live refreshing set of current set of
players in the lobby and proposed games that have not yet started. Games can be created, joined,
and left from this page.

### `GET /lobby/subscribe`

Returns a `text/event-stream` of events in the lobby. The following events can be returned in the
lobby event stream.
  
#### JoinLobby

Whenever a client subscribes to the lobby, a `join_lobby` message is sent to every other subscriber
in the lobby.

```json
{
  "type": "join_lobby",
  "player": "twilson"
}
```

#### LobbyState

Whenever a client subscribes to the lobby, a `lobby_state` message is sent to that subscriber
containing the list of all active subscribers, as well as all incomplete games.

```json
{
  "type": "lobby_state",
  "subscribers": ["tslatcher","twilson"],
  "games": {
    "8c9e2ff7-dcf3-49be-86f0-315f469840bc": ["carrino"]
  }
}
```

#### NewGame

Whenever a new game is created, a `new_game` message is sent to all active subscribers containg the
id of the game and the name of the player who created the game.

```json
{
  "type": "new_game",
  "id": "8c9e2ff7-dcf3-49be-86f0-315f469840bc",
  "player": "carrino"
}
```

#### JoinGame

Whenever a player joins an existing game, a `join_game` message is sent to all active subscribers
containing the id of the game and the name of the player who joined the game.

```json
{
  "type": "join_game",
  "id": "8c9e2ff7-dcf3-49be-86f0-315f469840bc",
  "player": "dcervelli"
}
```

#### LeaveGame

Whenever a player leaves an existing game, a `leave_game` message is sent to all active subscribers
containing the id of the game and the name of the player who left.

```json
{
  "type": "leave_game",
  "id": "8c9e2ff7-dcf3-49be-86f0-315f469840bc",
  "player": "carrino"
}
```

#### FinishGame

When the last play is made in a game a `finish_game` event is sent to all subscribers.

```json
{
  "type": "finish_game",
  "id": "8c9e2ff7-dcf3-49be-86f0-315f469840bc"
}
```

#### LeaveLobby

Whenever a player disconnects from the lobby every stream, a `leave_lobby` message is sent to all
other active subscribers.

```json
{
  "type": "leave_lobby",
  "player": "carrino"
}
```

### `POST /lobby/new`

Create a new game with the proposed charging rules and return its id. The actual charging rules
will be selected randomly from the proposed rules of all players once the game has four players.

Request:
```json
{
  "rules": "blind"
}
```

Response:
```json
"8c9e2ff7-dcf3-49be-86f0-315f469840bc"
```

### `POST /lobby/join`

Join an existing game and propose charging rules. Returns the members of the game. The actual
charging rules will be selected randomly from the proposed rules of all players once the game has
four players.

Request:
```json
{
  "id": "8c9e2ff7-dcf3-49be-86f0-315f469840bc",
  "rules": "chain"
}
```

Response:
```json
[
  "carrino",
  "dcervelli"
]
```

### `POST /lobby/add_bot`

Add a bot to an existing game and propose charging rules. Returns the name of the bot.

Request:
```json
{
  "id": "8c9e2ff7-dcf3-49be-86f0-315f469840bc",
  "rules": "chain",
  "algorithm": "random"
}
```

Response:
```json
"dharper (bot)"
```

### `POST /lobby/leave`

Leave an existing game.

Request:
```json
{
  "id": "8c9e2ff7-dcf3-49be-86f0-315f469840bc"
}
```

### `GET /game#<id>`

Returns an html page for a game displaying the live refreshing game state.

### `GET /game/subscribe/<id>`

Returns a `text/event-stream` of events in the game. The event stream will immediately return all
events that have occurred so far when it is connected. The event stream will be closed shortly
after the game is complete. Players not in the game can subscribe to the game as well to get an
unredacted stream of all events.

The following events can be returned in the game event stream.

#### Sit

When a game starts, a `sit` event is be sent indicating where each player is sitting and what the
charging rules are. The charging rules will be the rules proposed by the player in the `north`
seat.

```json
{
  "type": "sit",
  "north": {"type": "human", "name": "carrino"},
  "east": {"type": "human", "name": "tslatcher"},
  "south": {"type": "human", "name": "twilson"},
  "west": {"type": "bot", "name": "hjarvis (bot)", "algorithm": "random"},
  "rules": "blind"
}
```

#### Deal

When a new hand starts, a `deal` event is sent indicating which cards were dealt to which players.
Players in the game will receive a redacted event containing only their cards.

```json
{
  "type": "deal",
  "north": ["AS", "JS", "3S", "2S", "4H", "QC", "TC", "5C", "KD", "QD", "JD", "9D", "7D"],
  "east": ["QS", "TS", "6S", "5S", "4S", "AH", "JH", "TH", "9H", "8H", "2C", "AD", "3D"],
  "south": ["7S", "KH", "QH", "6H", "5H", "AC", "9C", "7C", "6C", "3C", "TD", "4D", "2D"],
  "west": ["KS", "9S", "8S", "7H", "3H", "2H", "KC", "JC", "8C", "4C", "8D", "6D", "5D"],
  "pass": "across"
}
```

#### SendPass

When a player makes a pass, a `send_pass` event is sent indicating who sent the pass and what cards
were passed. Players in the game other than the sender will receive a redacted event without the
actual cards passed.

```json
{
  "type": "send_pass",
  "from": "south",
  "cards": ["QH", "AC", "TD"]
}
```

#### RecvPass

When a player receives a pass, a `recv_pass` event is sent indicating who received the pass and
what cards they received. Players in the game other than the receiver will receive a redacted event
without the actual cards passed.

```json
{
  "type": "recv_pass",
  "to": "west",
  "cards": ["QH", "AC", "TD"]
}
```

#### Charge

When a charge is made (including an empty charge), a `charge` event is sent indicating who made the
charge and what cards they charged. If the charging rules use blind charges, players in the game
other than the charger will receive a `blind_charge` event instead.

```json
{
  "type": "charge",
  "seat": "east",
  "cards": ["QS", "AH"]
}
```

#### BlindCharge

When a blind variant of the charging rules has been chosen and a charge is made (including an empty
charge), a `blind_charge` event will be sent to other players in the game (the charger will receive
a `charge` event) indicating who made the charge and how many cards they charged.

```json
{
  "type": "blind_charge",
  "seat": "north",
  "count": 1
}
```

#### RevealCharges

When a blind variant of the charging rules has been chosen and a round of charging completes, a
`reveal_charges` event will be sent indicating what charges were made.

```json
{
  "type": "reveal_charges",
  "north": ["JD"],
  "east": [],
  "south": ["QS", "TC"],
  "west": []
}
```

#### Play

When a play is made, a `play` event will be sent indicating who made the play, and what card they
played.

```json
{
  "type": "play",
  "seat": "west",
  "card": "8D"
}
```

#### Start Trick

When a new trick starts, a `start_trick` event will be sent indicating which player makes the lead.
This event is sent for convenience; the information is imparts can be inferred from other events.

Response:
```json
{
  "type": "start_trick",
  "leader": "north"
}
```

#### End Trick

When a trick is completed, an `end_trick` event will be sent indicating which player makes won the
trick. This event is sent for convenience; the information is imparts can be inferred from other
events.

Response:
```json
{
  "type": "end_trick",
  "winner": "west"
}
```

### `POST /game/pass`

Pass cards.

Request:
```json
{
  "id": "8c9e2ff7-dcf3-49be-86f0-315f469840bc",
  "cards": ["QH", "AC", "TD"]
}
```

### `POST /game/charge`

Charge some cards. With chain style charging rules, all players must make a final empty charge to
complete the charging phase of a hand. Otherwise, all players other than the last to make a
non-empty charge must make a final empty charge to complete the charging phase.

Request:
```json
{
  "id": "8c9e2ff7-dcf3-49be-86f0-315f469840bc",
  "cards": ["QS", "AH"]
}
```

### `POST /game/play`

Play a card.

Request:
```json
{
  "id": "8c9e2ff7-dcf3-49be-86f0-315f469840bc",
  "card": "8D"
}
```
