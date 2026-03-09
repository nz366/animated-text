document.addEventListener('DOMContentLoaded', () => {
    const button = document.getElementById('clickMe');
    const message = document.getElementById('message');

    // YouTube Controls
    const playPauseBtn = document.getElementById('playPauseBtn');
    const rewindBtn = document.getElementById('rewindBtn');
    const forwardBtn = document.getElementById('forwardBtn');
    const videoIdDisplay = document.getElementById('videoIdDisplay');
    const currentTimeDisplay = document.getElementById('currentTimeDisplay');

    function formatTime(seconds) {
        if (!seconds) return "0:00";
        const h = Math.floor(seconds / 3600);
        const m = Math.floor((seconds % 3600) / 60);
        const s = Math.floor(seconds % 60);
        return [h, m, s]
            .filter((v, i) => v > 0 || i > 0)
            .map(v => v.toString().padStart(2, '0'))
            .join(':').replace(/^0/, '');
    }

    const wsStatus = document.getElementById('wsStatus');
    let socket = null;

    function setupWebSocket() {
        socket = new WebSocket('ws://127.0.0.1:3000/ws');

        socket.onopen = () => {
            console.log('Connected to remote-music-control server');
            wsStatus.innerText = 'Connected';
            wsStatus.classList.add('connected');
            wsStatus.classList.remove('disconnected');
        };

        socket.onmessage = (event) => {
            const command = event.data;
            console.log('Received command from server:', command);
            if (['play', 'pause', 'stop', 'forward', 'rewind'].includes(command)) {
                sendCommand(command).then(updateStatus);
            }
        };

        socket.onclose = () => {
            console.log('Disconnected from remote-music-control server');
            wsStatus.innerText = 'Disconnected';
            wsStatus.classList.add('disconnected');
            wsStatus.classList.remove('disconnected');
            // Try to reconnect after 3 seconds
            setTimeout(setupWebSocket, 3000);
        };

        socket.onerror = (error) => {
            console.error('WebSocket error:', error);
            socket.close();
        };
    }

    async function sendCommand(command, fromUser = false) {
        // If it's a user action, also send to WebSocket so server knows
        if (fromUser && socket && socket.readyState === WebSocket.OPEN) {
            socket.send(command);
        }

        const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
        if (tab?.url?.includes('youtube.com')) {
            try {
                const response = await chrome.tabs.sendMessage(tab.id, { command });
                return response;
            } catch (err) {
                console.error('Error sending message:', err);
            }
        }
        return null;
    }

    async function updateStatus() {
        const status = await sendCommand('getStatus', false);
        if (status && status.videoId) {
            videoIdDisplay.innerText = `Video: ${status.videoId}`;
            currentTimeDisplay.innerText = formatTime(status.currentTime);
            playPauseBtn.innerText = status.paused ? '▶️' : '⏸️';
        } else {
            videoIdDisplay.innerText = "No YouTube video detected";
            currentTimeDisplay.innerText = "--:--";
        }
    }

    // Initial status update
    updateStatus();
    // Poll for status updates
    const statusInterval = setInterval(updateStatus, 1000);
    // Initialize WebSocket
    setupWebSocket();

    playPauseBtn.addEventListener('click', async () => {
        const status = await sendCommand('getStatus', false);
        if (status) {
            await sendCommand(status.paused ? 'play' : 'pause', true);
            updateStatus();
        }
    });

    rewindBtn.addEventListener('click', () => sendCommand('rewind', true).then(updateStatus));
    forwardBtn.addEventListener('click', () => sendCommand('forward', true).then(updateStatus));

    // Magic Button Logic
    button.addEventListener('click', () => {
        message.classList.toggle('hidden');
        message.classList.toggle('animate-fade-in');

        if (!message.classList.contains('hidden')) {
            button.innerText = "Hide Magic";
            // Optional: send a message to server
            if (socket && socket.readyState === WebSocket.OPEN) {
                socket.send('magic_clicked');
            }
            console.log("Popup button clicked!");
        } else {
            button.innerText = "Click for Magic";
        }
    });
});
