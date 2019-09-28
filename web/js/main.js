$(".bet").click(function() {
    $("#popup").toggle();
});

$("#popup-ok").click(function() {
    $("#popup").toggle();
});

$("#popup-cancel").click(function() {
    $("#popup").toggle();
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


class Card {
    constructor(suit, number) {
        this.suit = suit;
        this.number = number;
    }
}
