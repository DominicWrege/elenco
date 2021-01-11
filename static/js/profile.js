import { updateFeed } from "./util.js";


window.addEventListener("load", () => {
    for (const element of document.querySelectorAll("img.action-offline")) {
        element.addEventListener("click", changeFeedOnline);
    }
    for (const element of document.querySelectorAll("img.action-online")) {
        element.addEventListener("click", changeFeedOffline);
    }
});


async function changeFeedOffline(event) {
    await genericUpdater(event, "Offline");
}


async function changeFeedOnline(event) {
    await genericUpdater(event, "Online");
}

async function genericUpdater(event, action) {
    if (confirm("Are you Sure?")) {
        try {
            const text = event.target.parentElement.querySelector("div.feed-id").textContent;
            const id = parseInt(text, 10);
            await updateFeed(id, action, "profile/update-feed");
            location.reload();
        } catch (err) {
            console.log(err);
        }
    }
}
