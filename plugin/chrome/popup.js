document.getElementById('grab').addEventListener('click', function () {
    chrome.tabs.query({ active: true, currentWindow: true }, function (tabs) {
        chrome.tabs.sendMessage(tabs[0].id, { message: "get_m3u8_urls" }, function (response) {
            let textToCopy = response.m3u8_url + " " + response.title.replace(" ", "").replaceAll(" ", "");
            navigator.clipboard.writeText(textToCopy).then(function () {
                console.log('Copying to clipboard was successful!');
                document.getElementById('info').textContent = 'copy success';
            }, function (err) {
                console.error('Could not copy text: ', err);
            });
        });
    });
});