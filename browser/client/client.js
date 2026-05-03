const dropzone = document.getElementById('dropzone');
const audioPlayer = document.getElementById('audio-player');
const fileInfo = document.getElementById('file-info');
const statusDot = document.getElementById('status-dot');
const statusText = document.getElementById('status-text');

let socket;

// Connect to WebSocket
function connect() {
    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    socket = new WebSocket(`${protocol}//${window.location.host}/ws`);

    socket.onopen = () => {
        statusDot.classList.add('connected');
        statusText.textContent = 'Connected to server';
        console.log('Connected to server');
    };

    socket.onclose = () => {
        statusDot.classList.remove('connected');
        statusText.textContent = 'Disconnected from server. Retrying...';
        console.log('Disconnected from server. Retrying in 3s...');
        setTimeout(connect, 3000);
    };

    socket.onmessage = (event) => {
        let request;
        try {
            request = JSON.parse(event.data);
        } catch (e) {
            request = { command: event.data };
        }

        console.log("Received command:", request);

        switch (request.command) {
            case "play":
                audioPlayer.play().catch(e => console.error("Play failed:", e));
                break;
            case "pause":
                audioPlayer.pause();
                break;
            case "seek":
                if (request.time !== undefined) {
                    audioPlayer.currentTime = request.time;
                }
                break;
            case "stop":
                audioPlayer.pause();
                audioPlayer.currentTime = 0;
                break;
        }
    };

    socket.onerror = (error) => {
        console.error('WebSocket error:', error);
    };
}

// 🔥 Call connect OUTSIDE the function
connect();


// Drag & Drop Logic
dropzone.addEventListener('dragover', (e) => {
    e.preventDefault();
    dropzone.classList.add('dragover');
});

dropzone.addEventListener('dragleave', () => {
    dropzone.classList.remove('dragover');
});

dropzone.addEventListener('drop', (e) => {
    e.preventDefault();
    dropzone.classList.remove('dragover');

    const files = e.dataTransfer.files;
    if (files.length > 0) {
        loadFile(files[0]);
    }
});

function loadFile(file) {
    if (!file.type.startsWith('audio/')) {
        alert('Please drop an audio file!');
        return;
    }

    const url = URL.createObjectURL(file);
    audioPlayer.src = url;
    fileInfo.textContent = `Loaded: ${file.name}`;
    console.log('File loaded:', file.name);
}


// Prevent spacebar from scrolling
window.addEventListener('keydown', (e) => {
    if (e.code === 'Space' && e.target === document.body) {
        e.preventDefault();
    }
});