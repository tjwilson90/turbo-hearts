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

function createAddBotButton() {
  let button = document.createElement("button");
  button.className = "add-bot";
  button.textContent = "Add Bot";
  button.addEventListener("click", addBot);
  return button;
}

function createJoinButton() {
  let button = document.createElement("button");
  button.className = "join";
  button.textContent = "Join";
  button.addEventListener("click", joinGame);
  return button;
}

function createLeaveButton() {
  let button = document.createElement("button");
  button.className = "leave";
  button.textContent = "Leave";
  button.addEventListener("click", leaveGame);
  return button;
}

function createOpenButton() {
  let button = document.createElement("button");
  button.className = "open";
  button.textContent = "Open";
  button.addEventListener("click", openGame);
  return button;
}

function getUserId() {
  return document.cookie.replace(
    /(?:(?:^|.*;\s*)USER_ID\s*\=\s*([^;]*).*$)|^.*$/,
    "$1"
  );
}

function subscribe() {
  if (eventStream != null) {
    eventStream.close();
  }
  eventStream = new EventSource("/lobby/subscribe");
  eventStream.onmessage = onEvent;
}

function newGame(event) {
  event.preventDefault();
  let rules = document.querySelector('input[name="rules"]:checked').value;
  console.log("newGame: %s", rules);
  fetch("/lobby/new", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({ rules: rules })
  })
    .then(response => response.json())
    .then(data => console.log("Created game: %o", data));
}

function addBot(event) {
  let game_id = event.target.parentNode.id;
  let rules = document.querySelector('input[name="rules"]:checked').value;
  console.log("addBot: %s, %s", game_id, rules);
  fetch("/lobby/add_bot", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({
      game_id: game_id,
      rules: rules,
      algorithm: algorithms[Math.floor(Math.random() * algorithms.length)]
    })
  })
  .then(response => response.json())
  .then(data => console.log("Add bot: %o", data));
}

function joinGame(event) {
  let game_id = event.target.parentNode.id;
  let rules = document.querySelector('input[name="rules"]:checked').value;
  console.log("joinGame: %s, %s", game_id, rules);
  fetch("/lobby/join", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({ game_id: game_id, rules: rules })
  })
  .then(response => response.json())
  .then(data => console.log("Joined game: %o", data));
}

function leaveGame(event) {
  let game_id = event.target.parentNode.id;
  console.log("leaveGame: %s", game_id);
  fetch("/lobby/leave", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({ game_id: game_id })
  });
}

function openGame(event) {
  let game_id = event.target.parentNode.id;
  console.log("openGame: %s", game_id);
  if (window.location.host.indexOf("localhost") !== -1) {
    window.open("http://localhost:8080#" + game_id);
  } else {
    window.open("https://play.anti.run/game#" + game_id);
  }
}

document.addEventListener("DOMContentLoaded", event => {
  document.getElementById("new-game").addEventListener("submit", newGame);
  subscribe();
});
