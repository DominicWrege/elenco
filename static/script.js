function showCustomAlter(message, type) {
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

window.addEventListener("load", () => {
    // showCustomAlter("This is an Alert!", "ok");
});