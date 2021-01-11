export function showCustomAlert(message, type) {
    const alertElement = document.querySelector("div#modal-alert");
    if (alertElement) {
        alertElement.textContent = message;
        alertElement.hidden = false;
        if (type === "error") {
            alertElement.classList.add("color-error");
        } else {
            alertElement.classList.add("color-ok");
        }
        setTimeout(() => {
            alertElement.hidden = !alertElement.hiiden;
        }, 3000);
    }
}


export async function updateFeed(id, action, path = "update-feed-status") {
    const options = {
        method: "PATCH",
        headers: {
            "Content-Type": "application/json",
        },
        body: JSON.stringify({ feed_id: id, action })
    };
    return fetch(path, options);

}

export default {}
