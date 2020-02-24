var eventStream = null;

function onEvent(event) {
  const data = JSON.parse(event.data);
  console.log("Event: %o", data);
  switch (data.type) {
    case "join_lobby":
      onJoinLobby(data.name);
      break;
    case "new_game":
      onNewGame(data.id, data.name);
      break;
    case "lobby_state":
      onLobbyState(data.subscribers, data.games);
      break;
    case "join_game":
      onJoinGame(data.id, data.player);
      break;
    case "leave_game":
      onLeaveGame(data.id, data.name);
      break;
    case "finish_game":
      onFinishGame(data.id);
      break;
    case "leave_lobby":
      onLeaveLobby(data.name);
      break;
    default:
      console.log("Unknown lobby event: %o", data);
      break;
  }
}

function onJoinLobby(name) {
  if (document.querySelector("#players > ." + name) == null) {
    let playerList = document.getElementById("players");
    playerList.appendChild(createPlayerElement(name));
  }
}

function onNewGame(id, name) {
  let gamesList = document.getElementById("games");
  gamesList.appendChild(createGameElement(id, [name]));
}

function onLobbyState(subscribers, games) {
  let playerList = document.getElementById("players");
  playerList.innerHTML = "";
  for (let name of subscribers) {
    playerList.appendChild(createPlayerElement(name));
  }
  let gamesList = document.getElementById("games");
  gamesList.innerHTML = "";
  for (let id in games) {
    let names = games[id].map(player => player.name);
    gamesList.appendChild(createGameElement(id, names));
  }
}

function onJoinGame(id, player) {
  let gameNode = document.getElementById(id);
  let playerList = gameNode.firstElementChild;
  playerList.appendChild(createPlayerElement(player.name));
  if (playerList.childElementCount >= 4) {
    gameNode.removeChild(gameNode.lastElementChild);
    gameNode.appendChild(createOpenButton());
  } else if (player.name === getName()) {
    gameNode.removeChild(gameNode.lastElementChild);
    gameNode.appendChild(createLeaveButton());
  }
}

function onLeaveGame(id, name) {
  let gameNode = document.getElementById(id);
  let playerList = gameNode.firstElementChild;
  for (playerNode of playerList.getElementsByClassName(name)) {
    playerNode.remove();
  }
  if (!playerList.hasChildNodes()) {
    gameNode.remove();
  } else if (name === getName()) {
    gameNode.removeChild(gameNode.lastElementChild);
    gameNode.appendChild(createJoinButton());
  }
}

function onFinishGame(id) {
  let gameNode = document.getElementById(id);
  if (gameNode != null) {
    gameNode.remove();
  }
}

function onLeaveLobby(name) {
  let playerNode = document.querySelector("#players" + "." + name);
  if (playerNode != null) {
    playerNode.remove();
  }
}

function createPlayerElement(name) {
  let li = document.createElement("li");
  li.className = name;
  li.textContent = name;
  return li;
}

function createGameElement(id, names) {
  let ul = document.createElement("ul");
  for (let name of names) {
    ul.appendChild(createPlayerElement(name));
  }
  let li = document.createElement("li");
  li.id = id;
  li.appendChild(ul);
  if (names.length >= 4) {
    li.appendChild(createOpenButton());
    return li;
  }
  li.appendChild(createAddBotButton());
  if (names.includes(getName())) {
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

function getName() {
  return document.cookie.replace(
    /(?:(?:^|.*;\s*)NAME\s*\=\s*([^;]*).*$)|^.*$/,
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
  let id = event.target.parentNode.id;
  let rules = document.querySelector('input[name="rules"]:checked').value;
  console.log("addBot: %s, %s", id, rules);
  fetch("/lobby/add_bot", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({ id: id, rules: rules, algorithm: "duck" })
  })
    .then(response => response.json())
    .then(data => console.log("Add bot: %o", data));
}

function joinGame(event) {
  let id = event.target.parentNode.id;
  let rules = document.querySelector('input[name="rules"]:checked').value;
  console.log("joinGame: %s, %s", id, rules);
  fetch("/lobby/join", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({ id: id, rules: rules })
  })
    .then(response => response.json())
    .then(data => console.log("Joined game: %o", data));
}

function leaveGame(event) {
  let id = event.target.parentNode.id;
  console.log("leaveGame: %s", id);
  fetch("/lobby/leave", {
    method: "POST",
    headers: {
      "Content-Type": "application/json"
    },
    body: JSON.stringify({ id: id })
  });
}

function openGame(event) {
  let id = event.target.parentNode.id;
  console.log("openGame: %s", id);
  if (window.location.host.indexOf("localhost") !== -1) {
    window.open("http://localhost:8080#" + id);
  } else {
    window.open("https://play.anti.run/assets/game/#" + id);
  }
}

document.addEventListener("DOMContentLoaded", event => {
  document.getElementById("new-game").addEventListener("submit", newGame);
  subscribe();
});
