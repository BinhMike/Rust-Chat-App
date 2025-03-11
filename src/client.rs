//! The client module manages user interaction and communication with the server.
//!
//! ## Overview
//! This module establishes a connection to the chat server, sends user input as messages,
//! and displays messages received from the server. It handles both broadcast and private
//! messages and tags the client's own messages with `(Me)` for clarity.
//!
//! ## Key Features
//! - Connects to the server and identifies as a unique client.
//! - Sends user input to the server for broadcasting or private messaging.
//! - Displays incoming messages in real-time, distinguishing private messages and self-messages.

use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

/// Starts the client and connects to the server.
///
/// This function establishes a connection to the server, reads the assigned client ID,
/// and spawns tasks to handle reading and writing messages. It facilitates interaction
/// between the user and the server.
///
/// # Arguments
/// * `address` - A string slice representing the server address (e.g., "127.0.0.1:8080").
///
/// # Errors
/// Returns an error if the connection to the server fails or if message processing encounters an issue.
///
/// # Example
/// ```no_run
/// use my_client::run_client;
///
/// #[tokio::main]
/// async fn main() {
///     run_client("127.0.0.1:8080").await.unwrap();
/// }
/// ```
pub async fn run_client(address: &str) -> std::io::Result<()> {
    // Establish a connection to the server
    let socket = TcpStream::connect(address).await?;
    let (reader, mut writer) = socket.into_split();
    let mut buf_reader = BufReader::new(reader);

    // Create a communication channel between tasks
    let (tx, mut rx) = tokio::sync::mpsc::channel::<String>(10);

    // Read and parse the client ID sent by the server
    let mut id_line = String::new();
    buf_reader.read_line(&mut id_line).await?;
    let my_id: usize = id_line
        .trim()
        .strip_prefix("Your ID: ")
        .unwrap()
        .parse()
        .unwrap();

    println!("Connected as Client {}", my_id);

    // Task to handle incoming messages from the server
    let read_task = tokio::spawn(async move {
        let mut line = String::new();
        while let Ok(bytes_read) = buf_reader.read_line(&mut line).await {
            if bytes_read == 0 {
                break; // Server connection closed
            }

            // Display private messages with a "[Private]" tag
            if line.contains("[Private]") {
                println!("{}", line.trim());
            } 
            // Tag the client's own messages with "(Me)"
            else if line.contains(&format!("Client {}:", my_id)) {
                print!("{} (Me)\n", line.trim());
            } 
            // Display all other messages as received
            else {
                print!("{}", line);
            }

            line.clear();
        }
    });

    // Task to handle user input from the terminal
    let tx_clone = tx.clone();
    let input_task = tokio::spawn(async move {
        let stdin = tokio::io::stdin();
        let mut lines = BufReader::new(stdin).lines();

        // Read user input line by line and send it to the server
        while let Ok(Some(line)) = lines.next_line().await {
            tx_clone.send(line).await.unwrap();
        }
    });

    // Main loop to send user messages to the server
    while let Some(message) = rx.recv().await {
        writer
            .write_all(format!("{}\n", message).as_bytes())
            .await?;
    }

    // Await the completion of the read and input tasks
    read_task.await.unwrap();
    input_task.await.unwrap();

    Ok(())
}