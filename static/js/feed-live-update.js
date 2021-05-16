
// const html_tr_str = `<tr>
// <td class="checkbox-container">
//     <input type="checkbox" class="feed-row pure-checkbox">
// </td>
// <td class="feed-id">11</td>
// <td><a href="/web/auth/feed/11">Bits und so</a></td>
// <td>
//     <a href="http://www.bitsundso.de/feed">Feed</a>
// </td>
// <td>Test</td>
// <td>
//     user123
// </td>
// <td>

//             <a href="http://www.bitsundso.de/">http://www.bitsundso.de/</a>

// </td>
// <td>
//     07:52, 23.04.21
// </td>
// </tr>`;

window.addEventListener("load", (e) => {
    console.log("works!");
    const table = document.querySelector("table.pure-table > tbody"); // giv table an ID
    const feedSocket = new WebSocket(`ws://${window.location.hostname}:${window.location.port}/web/auth/admin/fedd-live-update`);
    console.log(feedSocket);

    feedSocket.addEventListener("open", (e) => {
        if (feedSocket.readyState === 1) {
            feedSocket.addEventListener("message", (event) => {
                // console.log(event);
                if (event?.data) {
                    console.log(event.data);
                    table.insertAdjacentHTML("beforebegin", event.data);
                }

            });
        }
    });
});



//tbody.insertAdjacentHTML("beforebegin", html_tr_str);