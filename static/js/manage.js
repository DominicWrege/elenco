const feedsReviewed = new Set();

async function approvehandler(event) {
    await genericHandler("Online");
}

async function rejectHandler(event) {
    await genericHandler("Blocked");
}

async function genericHandler(action) {
    try {
        let udpates = Array.from(feedsReviewed).map(id => updateFeed(id, action));
        await Promise.all(udpates);
        location.reload();
    } catch (err) {
        console.error(err);
    }
}

async function updateFeed(id, action) {
    const options = {
        method: "PATCH",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({ feed_id: id, action })
    };
    return fetch("update-feed-status", options);

}


function checkboxChanged(event) {
    const id = event.target.parentElement.nextElementSibling.textContent.trim();
    const idInt = parseInt(id, 10);
    if (event.target.checked) {
        feedsReviewed.add(idInt);
    } else {
        feedsReviewed.delete(idInt);
    }
}

window.addEventListener("load", () => {

    document.querySelector("button#allowButton").addEventListener("click", approvehandler);
    document.querySelector("button#rejectButton").addEventListener("click", rejectHandler);

    for (const checkbox of document.querySelectorAll("tr td input.feed-row")) {
        checkbox.addEventListener("click", checkboxChanged);
    }
});





