window.addEventListener("load", () => {
    for (const item of document.querySelectorAll("td div img.player-button-play")) {
        item.addEventListener("click", startPlayer)
    }
    for (const item of document.querySelectorAll("td div img.player-button-pause")) {
        item.addEventListener("click", pausePlayer)
    }
    const player = new Player();

    function startPlayer(event) {
        const target = event.target;

        const media_url = target.parentElement.querySelector("div.media-url").textContent;
        player.play(media_url, target.parentElement);
    }
    function pausePlayer(event) {
        player.pause(event.target);
    }
});


class Player {
    constructor() {
        this.audio = null;
        this.playingRow = null;
        this.isPlaying = false;
    }
    play(url, row) {
        if (this.isPlaying) {
            this.pause(this.playingRow.querySelector(".player-button-pause"));
        }
        row.querySelector(".player-button-pause").hidden = false;
        row.querySelector(".player-button-play").hidden = true;
        this.audio = new Audio(url);
        this.audio.play();
        this.playingRow = row;
        this.isPlaying = true;
    }
    pause(target) {
        target.hidden = true;
        this.audio.pause();
        this.isPlaying = false;
        this.playingRow.querySelector(".player-button-play").hidden = false;
    }
}
