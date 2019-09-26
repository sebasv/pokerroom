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
    let card = new Card("spades", 4);
    card.display($(".spades")[0]);
});


class Card {
    constructor(suit, number) {
        this.suit = suit;
        this.number = number;
    }

    display(element) {
        let offsetX = -this.number * 72;
        let offsetY = 0;
        if (this.suit == "hearts") {
            offsetY = 0;
        } else if (this.suit === "diamonds") {
            offsetY = -100;
        } else if (this.suit === "clubs") {
            offsetY = -200;
        } else if (this.suit === "spades") {
            offsetY = -300;
        }
        element.style.background = "url('img/playing_cards.gif') " + 
            offsetX + "px " + offsetY + "px";
    }
}
