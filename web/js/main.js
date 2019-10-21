// when page is finished loading
$(document).ready(function() {
    let nPlayers = 6;
    initializeRoom(nPlayers);


    let cards = [new Card("spades", 0), new Card("hearts", 0)];
    initializeRound(nPlayers, cards);
});




$(".bet").click(function() {
    $("#popup").toggle();
});

$("#popup-ok").click(function() {
    let betSize = $("#popup > .popup-body > input").value
    placeBet(betSize);
});

$("#popup-cancel").click(function() {
    cancelBet();
});

$(".call").click(function() {
    let card1 = new Card("spades", 0);
    let card2 = new Card("hearts", 0);
    let card3 = new Card("diamonds", 0);
    let card4 = new Card("clubs", 0);
    let card5 = new Card("spades", 10);

    showFlop([card1, card2, card3]);
    // showTurn(card4);
    // showRiver(card5);
    // dealCards(0, [card1, card2]);
    // setStatus(1, 2, "Lo")
});

function setPlayerChips(player, chips) {
    $("#player-" + player + "-chips").text(chips);
}

function setPlayerMsg(player, msg) {
    $("#player-" + player + "-status").text(msg);
}

function setStatus(player, chips, msg) {
    setPlayerChips(player, chips);
    setPlayerMsg(player, msg);
}

function displayCard(card, element) {
    let offsetX = -card.number * 72;
    let offsetY = 0;
    if (card.suit == "hearts") {
        offsetY = 0;
    } else if (card.suit === "diamonds") {
        offsetY = -100;
    } else if (card.suit === "clubs") {
        offsetY = -200;
    } else if (card.suit === "spades") {
        offsetY = -300;
    } else if (card.suit === "closed") {
        offsetX = -360;
        offsetY = -400;
    }
    element.style.background = "url('img/playing_cards.gif') " + 
        offsetX + "px " + offsetY + "px";
}

function dealCards(player, cards) {
    $("#player-" + player + " > .card_layer > .card").each(function(index) {
        displayCard(cards[index], this);
    });
}

function showFlop(cards) {
    $("#table_layer > div > div").each(function(index) {
        if (index > 2) {
            return false;
        }
        displayCard(cards[index], this);
    });
}

function showTurn(card) {
    $("#table_layer > .card_layer > .card").each(function(index) {
        if (index != 3) {
            return true;
        }
        displayCard(card, this);
    });
}

function showRiver(card) {
    $("#table_layer > .card_layer > .card").each(function(index) {
        if (index != 4) {
            return true;
        }
        displayCard(card, this);
    });
}

function placeBet(amount) {
    // talk to server
    // ...

    // clean up
    let inputs = $("#popup > .popup-body > input");
    inputs[0].value = "";
    $("#popup").toggle();
}

function cancelBet() {
    let inputs = $("#popup > .popup-body > input");
    inputs[0].value = "";
    $("#popup").toggle();
}

function appointDealer(player) {
    let playmarkers = $("div > .playmarker");
    
    playmarkers.removeClass("dealer");
    
    let playerMarker = $("#playmarker-" + player);
    playerMarker.addClass("dealer");
}

function initializeRoom(nPlayers) {
    for (let i = 1; i < nPlayers; i++) {
        let playerElement = createPlayerHTML(i);
        $("#opponents_layer").append(playerElement);
    }
}

function initializeRound(nPlayers, cards) {
    // deal cards to other players    
    let closedCard = new Card("closed", 0);
    for (let i = 1; i < nPlayers; i++) {
        dealCards(i, [closedCard, closedCard]);
        setStatus(i, 0, "");
    }

    // deal cards to main player
    dealCards(0, cards);
    setStatus(0, 0, "");

    // reset pot
    $("#pot-size").html("0");

    // appoint dealer
    appointDealer(0);
}

function createPlayerHTML(player) {
    let playerElement = $("<div></div>");
    playerElement.attr("id", "player-" + player);
    playerElement.addClass("player");

    let cardLayerElement = $("<div></div>");
    cardLayerElement.addClass("card_layer");
    cardLayerElement.append("<div class='card'></div>");
    cardLayerElement.append("<div class='card'></div>");
    playerElement.append(cardLayerElement);

    let table = $("<table></table>");
    let row1 = $("<tr></tr>");
    let td11 = $("<td>Chips:</td>");
    let td12 = $("<td>0</td>");
    td12.attr("id", "player-" + player);
    row1.append(td11);
    row1.append(td12);
    let row2 = $("<tr></tr>");
    let td21 = $("<td>Status:</td>");
    let td22 = $("<td></td>");
    td22.attr("id", "player-" + player);
    row2.append(td21);
    row2.append(td22);
    table.append(row1);
    table.append(row2);
    let playerStatusElement = $("<div class='playerstatus'></div>");
    playerStatusElement.append(table);
    
    let markerElement = $("<div class='playmarker'></div>");
    markerElement.attr("id", "playmarker-" + player);
    playerStatusElement.append(markerElement);

    playerElement.append(playerStatusElement);

    return playerElement;
}


class Card {
    constructor(suit, number) {
        this.suit = suit;
        this.number = number;
    }
}