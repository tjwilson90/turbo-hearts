# Turbo Hearts

## Endpoints

All endpoints require the caller to pass a `"player"` cookie identifying themselves. There's no
authentication; please don't cheat.

### `GET /lobby`

TODO: implement

Returns an html page for the game lobby displaying the live refreshing set of current set of
players in the lobby and proposed games that have not yet started. Games can be created, joined,
and left from this page.

### `GET /lobby/subscribe`

Returns a `text/event-stream` of events in the lobby. The following events can be returned in the
lobby event stream.
  
#### Subscribe

Whenever a client subscribes to the lobby, a `Subscribe` message is sent to every other subscriber
in the lobby.

Response:
```json
{
  "type": "Subscribe",
  "player": "twilson"
}
```

#### LobbyState

Whenever a client subscribes to the lobby, a `LobbyState` message is sent to that subscriber
containing the list of all active subscribers, as well as all partial games that need additional
players.

Response:
```json
{
  "type": "LobbyState",
  "subscribers": ["tslatcher","twilson"],
  "games": {
    "8c9e2ff7-dcf3-49be-86f0-315f469840bc": {
      "carrino": "Blind"
    }
  }
}
```

#### NewGame

Whenever a new game is created, a `NewGame` message is sent to all active subscribers containg the
id of the game, the name of the player who created the game, and the charging rules they proposed.

Response:
```json
{
  "type": "NewGame",
  "id": "8c9e2ff7-dcf3-49be-86f0-315f469840bc",
  "player": "carrino",
  "rules": "Blind"
}
```

#### JoinGame

Whenever a player joins an existing game, a `JoinGame` message is sent to all active subscribers
containing the id of the game, the name of the player who joined the game, and the charging rules
they proposed.

Response:
```json
{
  "type": "JoinGame",
  "id": "8c9e2ff7-dcf3-49be-86f0-315f469840bc",
  "player": "dcervelli",
  "rules": "Chain"
}
```

#### LeaveGame

Whenever a player leaves an existing game, a `LeaveGame` message is sent to all active subscribers
containing the id of the game and the name of the player who left.

Response:
```json
{
  "type": "LeaveGame",
  "id": "8c9e2ff7-dcf3-49be-86f0-315f469840bc",
  "player": "carrino"
}
```

#### LeaveLobby

Whenever a player disconnects from the lobby every stream, a `LeaveLobby` message is sent to all
other active subscribers.

Response:
```json
{
  "type": "LeaveLobby",
  "player": "carrino"
}
```

### `POST /lobby/new`

Create a new game with the proposed charging rules and return its id. The actual charging rules
will be selected randomly from the proposed rules of all players once the game has four players.

Request:
```json
{
  "rules": "Blind"
}
```

Response:
```json
"8c9e2ff7-dcf3-49be-86f0-315f469840bc"
```

### `POST /lobby/join`

Join an existing game and propose charging rules. Returns the members of the game and their
proposed charging rules. The actual charging rules will be selected randomly from the proposed
rules of all players once the game has four players.

Request:
```json
{
  "id": "8c9e2ff7-dcf3-49be-86f0-315f469840bc",
  "rules": "Chain"
}
```

Response:
```json
{
  "carrino": "Blind",
  "dcervelli": "Chain"
}
```

### `POST /lobby/leave`

Leave an existing game.

Request:
```json
{
  "id": "8c9e2ff7-dcf3-49be-86f0-315f469840bc"
}
```

### `GET /game/<id>`

TODO: implement

Returns an html page for a game displaying the live refreshing game state.

### `GET /game/subscribe/<id>`

Returns a `text/event-stream` of events in the game. The event stream will immediately return all
events that have occurred so far when it is connected. The event stream will be closed once the game
is complete.

All events have the form
```json
{
  "seat": "West",
  "timestamp": 1581006483853,
  "kind": { ... }
}
```
where the seat identifies  form

The following events can be returned in the game event stream.

#### ReceiveHand

When a new hand starts, 

Response:
```json
{
  "hand": ["AS", "JS", "3S", "2S", "4H", "QC", "TC", "5C", "KD", "QD", "JD", "9D", "7D"]
}
```
