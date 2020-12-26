function showCustomAltert(message, type) {
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
    highlightSeletcedMenu();
});

function highlightSeletcedMenu() {
    let path = location.pathname;
    if (path.indexOf("/web/auth/admin") === 0) {
        path = "/web/auth/admin/manage";
    }
    const link = document.querySelector(`ul.pure-menu-list a.pure-menu-link[href='${path}']`).parentElement;
    link.classList.add("pure-menu-selected");
}