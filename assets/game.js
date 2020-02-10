
function getName() {
    return document.cookie.replace(/(?:(?:^|.*;\s*)player\s*\=\s*([^;]*).*$)|^.*$/, '$1');
}

function getGameId() {
    let path = window.location.pathname;
    let index = path.lastIndexOf('/');
    return path.substring(index + 1);
}

function onEvent(event) {
    const data = JSON.parse(event.data);
    console.log('Event: %o', data);
    switch (data.type) {
        case 'sit':
            onSit(data.north, data.east, data.south, data.west, data.rules);
            break;
        case 'deal':
            onDeal(data.north, data.east, data.south, data.west);
            break;
        case 'send_pass':
            onSendPass(data.from, data.cards);
            break;
        case 'recv_pass':
            onRecvPass(data.to, data.cards);
            break;
        case 'blind_charge':
            onBlindCharge(data.seat, data.count);
            break;
        case 'charge':
            onCharge(data.seat, data.cards);
            break;
        case 'play':
            onPlay(data.seat, data.card, data.trick_number);
            break;
        default:
            console.log('Unknown lobby event: %o', data);
            break;
    }
}

function onSit(north, east, south, west, rules) {
    onSitPlayer(north, document.getElementById('north'));
    onSitPlayer(east, document.getElementById('east'));
    onSitPlayer(south, document.getElementById('south'));
    onSitPlayer(west, document.getElementById('west'));
    document.getElementById('rules').innerHTML = rules;
}

function onSitPlayer(player, div) {
    div.innerHTML = '';
    let h1 = document.createElement('h1');
    h1.append(player);
    div.appendChild(h1);
}

function onDeal(north, east, south, west) {
    addCards(north, document.getElementById('north'));
    addCards(east, document.getElementById('east'));
    addCards(south, document.getElementById('south'));
    addCards(west, document.getElementById('west'));
    document.getElementById('events').innerHTML = '';
}

function addCards(cards, div) {
    for (let card of cards) {
        div.appendChild(createCardElement(card));
    }
}

function onSendPass(from, cards) {
    for (let card of cards) {
        let elem = document.getElementById(card);
        if (elem != null) {
            elem.remove();
        }
    }
}

function onRecvPass(to, cards) {
    addCards(cards, document.getElementById(to));
}

function onBlindCharge(seat, count) {
    let p = document.createElement('p');
    p.append("Blind charge of " + count + " card(s) from " + seat);
    let events = document.getElementById('events');
    events.appendChild(p);
}

function onCharge(seat, cards) {
    let p = document.createElement('p');
    p.append("Charge of " + JSON.stringify(cards) + " from " + seat);
    let events = document.getElementById('events');
    events.appendChild(p);
}

function onPlay(seat, card, trick_number) {
    let p = document.createElement('p');
    p.className = 'trick-' + trick_number;
    p.append(seat + " played " + card);
    let events = document.getElementById('events');
    events.appendChild(p);
    let cardElement = document.getElementById(card);
    if (cardElement != null) {
        cardElement.remove();
    }
    let expired = document.getElementsByClassName('trick-' + (trick_number - 2));
    while (expired.length > 0) {
        expired[0].remove();
    }
}

function createCardElement(card) {
    let input = document.createElement('input');
    input.type = 'checkbox';
    input.name = 'card';
    input.value = card;
    let label = document.createElement('label');
    label.id = card;
    label.appendChild(input);
    label.append(card);
    return label;
}

function getCheckedCards() {
    let cardElements = document.querySelectorAll('input[name=card]:checked');
    let cards = [];
    for (let element of cardElements) {
        cards.push(element.value);
    }
    return cards;
}

function passCards() {
    let cards = getCheckedCards();
    console.log('passCards:, %o', cards);
    fetch("/game/pass", {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({ id: getGameId(), cards: cards })
    });
}

function chargeCards() {
    let cards = getCheckedCards();
    console.log('chargeCards:, %o', cards);
    fetch("/game/charge", {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({ id: getGameId(), cards: cards })
    });
}

function playCard() {
    let cards = getCheckedCards();
    console.log('playCard: %o', cards);
    fetch("/game/play", {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json'
        },
        body: JSON.stringify({ id: getGameId(), card: cards[0] })
    });
}

document.addEventListener('DOMContentLoaded', (event) => {
    document.getElementById('pass-cards').addEventListener('click', passCards);
    document.getElementById('charge-cards').addEventListener('click', chargeCards);
    document.getElementById('play-card').addEventListener('click', playCard);

    let eventStream = new EventSource('/game/subscribe/' + getGameId());
    eventStream.onmessage = onEvent;
});
