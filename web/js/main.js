// when page is finished loading
$(document).ready(function() {
    let nPlayers = 6;
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


class Card {
    constructor(suit, number) {
        this.suit = suit;
        this.number = number;
    }
}