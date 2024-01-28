
// background.js
chrome.webRequest.onBeforeRequest.addListener(
    function (details) {
        if (details.url.indexOf('.m3u8') > 0) {
            chrome.tabs.query({ active: true, currentWindow: true }, function (tabs) {
                chrome.tabs.sendMessage(tabs[0].id, { url: details.url });
            });
        }

    },
    { urls: ["<all_urls>"] }
);
