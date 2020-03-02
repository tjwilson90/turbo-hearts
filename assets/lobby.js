var eventStream = null;
let algorithms = ["duck", "gottatry", "random"];

function onEvent(event) {
  const data = JSON.parse(event.data);
  console.log("Event: %o", data);
  switch (data.type) {
    case "join_lobby":
      onJoinLobby(data.user_id);
      break;
    case "new_game":
      onNewGame(data.game_id, data.user_id);
      break;
    case "lobby_state":
      onLobbyState(data.subscribers, data.games);
      break;
    case "join_game":
      onJoinGame(data.game_id, data.player);
      break;
    case "leave_game":
      onLeaveGame(data.game_id, data.user_id);
      break;
    case "finish_game":
      onFinishGame(data.game_id);
      break;
    case "leave_lobby":
      onLeaveLobby(data.user_id);
      break;
    default:
      console.log("Unknown lobby event: %o", data);
      break;
  }
}

function onJoinLobby(user_id) {
  if (document.querySelector("#players > ." + user_id) == null) {
    let playerList = document.getElementById("players");
    playerList.appendChild(createPlayerElement(user_id));
  }
}

function onNewGame(game_id, user_id) {
  let gamesList = document.getElementById("games");
  gamesList.appendChild(createGameElement(game_id, [user_id]));
}

function onLobbyState(subscribers, games) {
  let playerList = document.getElementById("players");
  playerList.innerHTML = "";
  for (let user_id of subscribers) {
    playerList.appendChild(createPlayerElement(user_id));
  }
  let gamesList = document.getElementById("games");
  gamesList.innerHTML = "";
  for (let game_id in games) {
    let user_ids = games[game_id].map(player => player.user_id);
    gamesList.appendChild(createGameElement(game_id, user_ids));
  }
}

function onJoinGame(game_id, player) {
  let gameNode = document.getElementById(game_id);
  let playerList = gameNode.firstElementChild;
  playerList.appendChild(createPlayerElement(player.user_id));
  if (playerList.childElementCount >= 4) {
    gameNode.removeChild(gameNode.lastElementChild);
    gameNode.appendChild(createOpenButton());
  } else if (player.user_id === getUserId()) {
    gameNode.removeChild(gameNode.lastElementChild);
    gameNode.appendChild(createLeaveButton());
  }
}

function onLeaveGame(game_id, user_id) {
  let gameNode = document.getElementById(game_id);
  let playerList = gameNode.firstElementChild;
  for (playerNode of playerList.getElementsByClassName(user_id)) {
    playerNode.remove();
  }
  if (!playerList.hasChildNodes()) {
    gameNode.remove();
  } else if (user_id === getUserId()) {
    gameNode.removeChild(gameNode.lastElementChild);
    gameNode.appendChild(createJoinButton());
  }
}

function onFinishGame(game_id) {
  let gameNode = document.getElementById(game_id);
  if (gameNode != null) {
    gameNode.remove();
  }
}

function onLeaveLobby(user_id) {
  let playerNode = document.querySelector("#players" + "." + user_id);
  if (playerNode != null) {
    playerNode.remove();
  }
}

function createPlayerElement(user_id) {
  let li = document.createElement("li");
  li.className = user_id;
  li.textContent = user_id;
  return li;
}

function createGameElement(game_id, user_ids) {
  let ul = document.createElement("ul");
  for (let user_id of user_ids) {
    ul.appendChild(createPlayerElement(user_id));
  }
  let li = document.createElement("li");
  li.id = game_id;
  li.appendChild(ul);
  if (user_ids.length >= 4) {
    li.appendChild(createOpenButton());
    return li;
  }
  li.appendChild(createAddBotButton());
  if (user_ids.includes(getUserId())) {
    li.appendChild(createLeaveButton());
  } else {
    li.appendChild(createJoinButton());
  }
  return li;
}

function getUserId() {
  return document.cookie.replace(
    /(?:(?:^|.*;\s*)USER_ID\s*\=\s*([^;]*).*$)|^.*$/,
    "$1"
  );
}

function openGame(event) {
  let game_id = event.target.parentNode.id;
  console.log("openGame: %s", game_id);
  window.open("/game/#" + game_id);
}

document.addEventListener("DOMContentLoaded", event => {
  document.getElementById("new-game").addEventListener("submit", newGame);
  subscribe();
});
