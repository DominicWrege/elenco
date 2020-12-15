async function approvehandler(event) {
    await genericHandler(event, "Online");
}


async function rejectHandler(event) {
    await genericHandler(event, "Blocked");
}

async function genericHandler(event, action) {
    const id = extractId(event.target)
    try {
        await updateFeed(id, action);
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


function extractId(target) {
    const idElement = target.parentElement.parentElement.querySelector(".feed-id");
    return parseInt(idElement.textContent.trim(), 10);
}

window.addEventListener("load", () => {
    for (const button of document.querySelectorAll("td.action button.allow")) {
        button.addEventListener("click", approvehandler);
    }
    for (const button of document.querySelectorAll("td.action button.reject")) {
        button.addEventListener("click", rejectHandler);
    }
});





