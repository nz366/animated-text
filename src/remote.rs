use axum::{
    extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State},
    routing::get,
    Router,
};
use futures_util::{SinkExt, StreamExt};
use std::net::SocketAddr;
use tokio::sync::broadcast;
use tower_http::services::ServeDir;

pub fn start_server(port: u16) -> broadcast::Sender<String> {
    // Create a broadcast channel for commands
    let (tx, _rx) = broadcast::channel::<String>(16);
    let tx_clone = tx.clone();

    tokio::spawn(async move {
        // Build our application with a route
        let app = Router::new()
            // Extract `tx` from the application State
            .route("/ws", get(|ws: WebSocketUpgrade, State(tx): State<broadcast::Sender<String>>| async move {
                ws.on_upgrade(move |socket| handle_socket(socket, tx))
            }))
            .with_state(tx_clone.clone())
            .fallback_service(ServeDir::new(concat!(env!("CARGO_MANIFEST_DIR"), "/browser/client")));

        // Run it
        let addr = SocketAddr::from(([127, 0, 0, 1], port));
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        let _ = tx_clone.send(format!("Listening on {}", addr));
        axum::serve(listener, app).await.unwrap();
    });

    tx
}

async fn handle_socket(socket: WebSocket, tx: broadcast::Sender<String>) {
    let (mut sender, mut receiver) = socket.split();
    let mut rx = tx.subscribe();

    let _ = tx.send("Connected".to_string());

    // Task for sending broadcast messages to the websocket
    let mut send_task = tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if sender.send(Message::Text(msg)).await.is_err() {
                break;
            }
        }
    });

    // Task for receiving messages from the websocket (and keeping it alive)
    let tx_for_recv = tx.clone();
    let mut recv_task = tokio::spawn(async move {
        while let Some(Ok(msg)) = receiver.next().await {
            match msg {
                Message::Text(text) => {
                    let _ = tx_for_recv.send(text);
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    });

    // If any task finishes, abort the other one
    tokio::select! {
        _ = (&mut send_task) => recv_task.abort(),
        _ = (&mut recv_task) => send_task.abort(),
    };
    let _ = tx.send("Disconnected".to_string());
}
