var m3u8_url = ""

// content.js
chrome.runtime.onMessage.addListener(
    function (request, sender, sendResponse) {
        if (request.url != undefined) {
            m3u8_url = request.url;
        } else {
            sendResponse({ m3u8_url: m3u8_url, title: document.title });
        }
    }
);