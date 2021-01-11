window.addEventListener("load", () => {
    for (const item of document.querySelectorAll("td div img.player-button-play")) {
        item.addEventListener("click", startPlayer)
    }
    for (const item of document.querySelectorAll("td div img.player-button-pause")) {
        item.addEventListener("click", pausePlayer)
    }
    const player = new AudioPlayer();

    async function startPlayer(event) {
        const target = event.target;

        const media_url = target.parentElement.querySelector("div.media-url").textContent;
        await player.play(media_url, target.parentElement);
    }
    function pausePlayer() {
        player.pause();
    }


    navigator.mediaSession.setActionHandler("play", async () => {
        await player.resume();
    });
    navigator.mediaSession.setActionHandler("pause", () => {
        player.pause();
    });
    navigator.mediaSession.setActionHandler("seekforward", (event) => {
        player.seekForward();
    });
    navigator.mediaSession.setActionHandler("seekbackward", (event) => {
        player.seekBackforward();
    });
});


class AudioPlayer {
    static skipTime = 30;
    constructor() {
        this.audio = null;
        this.playingRow = null;
        this.isPlaying = false;
    }
    async play(url, row) {
        if (this.isPlaying) {
            this.pause();
        }
        row.querySelector(".player-button-pause").hidden = false;
        row.querySelector(".player-button-play").hidden = true;
        navigator.mediaSession.metadata = new MediaMetadata({
            title: row.querySelector(".episode-title").textContent,
            artist: document.querySelector(".podcast-header-form #podcast-title").textContent,
        });
        this.audio = new Audio(url);
        await this.audio.play();
        this.playingRow = row;
        this.isPlaying = true;
    }
    async resume() {
        if (!this.isPlaying) {
            this.isPlaying = true;
            this.playingRow.querySelector(".player-button-play").hidden = true;
            this.playingRow.querySelector(".player-button-pause").hidden = false;
            await this.audio.play();
        };
    }
    pause() {
        this.audio.pause();
        this.isPlaying = false;
        this.playingRow.querySelector(".player-button-play").hidden = false;
        this.playingRow.querySelector(".player-button-pause").hidden = true;
    }
    seekForward() {
        this.audio.currentTime = this.audio.currentTime + AudioPlayer.skipTime;
    }
    seekBackforward() {
        this.audio.currentTime = Math.max(this.audio.currentTime - AudioPlayer.skipTime, 0);
    }

}
