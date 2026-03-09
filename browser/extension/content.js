// content.js
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
    const video = document.querySelector('video');

    if (!video) {
        sendResponse({ error: 'No video element found' });
        return;
    }

    switch (request.command) {
        case 'play':
            video.play();
            break;
        case 'pause':
            video.pause();
            break;
        case 'rewind':
            video.currentTime -= 10;
            break;
        case 'forward':
            video.currentTime += 10;
            break;
        case 'getStatus':
            const urlParams = new URLSearchParams(window.location.search);
            const videoId = urlParams.get('v');
            sendResponse({
                paused: video.paused,
                currentTime: video.currentTime,
                videoId: videoId
            });
            return true; // Keep channel open for async response
    }

    sendResponse({ status: 'success' });
});
