//! The server module handles incoming client connections and broadcasts messages.
//!
//! ## Overview
//! This module provides the core functionality for a simple chat server that:
//! - Handles multiple client connections concurrently.
//! - Allows clients to broadcast messages to all connected users.
//! - Enables private messaging between specific clients.
//! - Manages disconnections and ensures the server continues functioning even if a client disconnects.
//!
//! ## Key Features
//! - **Broadcast Messaging**: Messages sent by a client are broadcasted to all connected clients.
//! - **Private Messaging**: Clients can send private messages using the `/msg <client_id> <message>` command.
//! - **Concurrency**: Uses Tokio's asynchronous features to handle multiple clients concurrently.
//! - **Graceful Disconnection**: Removes disconnected clients from the shared client list without crashing the server.

use std::sync::Arc;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpListener,
    sync::Mutex,
};

/// A thread-safe, shared collection of client connections.
///
/// Each client connection is represented by a `tokio::net::tcp::OwnedWriteHalf`,
/// which allows sending messages to the client.
type SharedClients = Arc<Mutex<Vec<tokio::net::tcp::OwnedWriteHalf>>>;

/// Starts the server and listens for incoming connections.
///
/// This function initializes the server, binds to the provided address,
/// and waits for client connections. For each connected client, it spawns
/// a new task to handle the connection.
///
/// # Arguments
/// - `address`: A string slice representing the IP address and port to bind to (e.g., `"127.0.0.1:8080"`).
///
/// # Errors
/// Returns an error if the server fails to bind to the address.
///
/// # Example
/// ```no_run
/// use my_server::run_server;
///
/// #[tokio::main]
/// async fn main() {
///     run_server("127.0.0.1:8080").await.unwrap();
/// }
/// ```
pub async fn run_server(address: &str) -> std::io::Result<()> {
    let listener = TcpListener::bind(address).await?;
    println!("Server listening on {}", address);

    let clients: SharedClients = Arc::new(Mutex::new(Vec::new()));
    let mut client_id = 1;

    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection: {} (Client {})", addr, client_id);

        let (reader, mut writer) = socket.into_split();
        let clients = clients.clone();

        let current_id = client_id;
        client_id += 1;

        writer
            .write_all(format!("Your ID: {}\n", current_id).as_bytes())
            .await?;

        tokio::spawn(async move {
            handle_connection(reader, writer, clients, current_id).await;
        });
    }
}

/// Handles an individual client connection.
///
/// This function processes client messages and determines whether they should be
/// broadcast to all clients or sent privately to a specific client. It also removes
/// the client from the shared list upon disconnection.
///
/// # Arguments
/// - `reader`: A read handle for the client connection.
/// - `writer`: A write handle for the client connection.
/// - `clients`: A shared collection of all connected clients.
/// - `client_id`: A unique identifier for the client.
async fn handle_connection(
    reader: tokio::net::tcp::OwnedReadHalf,
    writer: tokio::net::tcp::OwnedWriteHalf,
    clients: SharedClients,
    client_id: usize,
) {
    let mut buf_reader = BufReader::new(reader);
    let mut line = String::new();

    {
        // Add the client to the shared list
        clients.lock().await.push(writer);
    }

    while let Ok(bytes_read) = buf_reader.read_line(&mut line).await {
        if bytes_read == 0 {
            break; // Client disconnected
        }

        let trimmed_line = line.trim();
        if let Some((target_id, private_msg)) = parse_private_message(trimmed_line) {
            let message = format!("[Private] Client {}: {}", client_id, private_msg);
            println!(
                "Private message from Client {} to Client {}: {}",
                client_id, target_id, private_msg
            );

            send_private_message(clients.clone(), target_id, &message).await;
        } else {
            let message = format!("Client {}: {}", client_id, trimmed_line);
            println!("{}", message);

            broadcast_message(clients.clone(), &message).await;
        }

        line.clear();
    }

    println!("Client {} disconnected.", client_id);
}

/// Parses a private message command.
///
/// This function interprets a message with the `/msg` command format.
/// Valid commands are of the format `/msg <client_id> <message>`.
///
/// # Arguments
/// - `input`: The command string to parse.
///
/// # Returns
/// - `Some((client_id, message))` if the input is valid.
/// - `None` if the input is invalid.
///
/// # Example
/// ```
/// let result = parse_private_message("/msg 2 Hello!");
/// assert_eq!(result, Some((2, "Hello!")));
/// ```
fn parse_private_message(input: &str) -> Option<(usize, &str)> {
    if input.starts_with("/msg ") {
        let parts: Vec<&str> = input.splitn(3, ' ').collect();
        if parts.len() == 3 {
            if let Ok(target_id) = parts[1].parse::<usize>() {
                return Some((target_id, parts[2]));
            }
        }
    }
    None
}

/// Sends a private message to a specific client.
///
/// Retrieves the specified client by ID and sends the provided message. If the client
/// does not exist or the message fails to send, it logs an error.
///
/// # Arguments
/// - `clients`: A shared collection of all connected clients.
/// - `target_id`: The ID of the target client.
/// - `message`: The message to send.
///
/// # Errors
/// Logs an error if the client does not exist or the message fails to send.
async fn send_private_message(clients: SharedClients, target_id: usize, message: &str) {
    let mut clients = clients.lock().await;
    if let Some(writer) = clients.get_mut(target_id - 1) {
        if writer
            .write_all(format!("{}\n", message).as_bytes())
            .await
            .is_err()
        {
            println!("Failed to send private message to Client {}", target_id);
        }
    } else {
        println!("Client {} not found.", target_id);
    }
}

/// Broadcasts a message to all connected clients.
///
/// Sends the message to every client in the shared list. If a client
/// is unreachable, it is removed from the list.
///
/// # Arguments
/// - `clients`: A shared collection of all connected clients.
/// - `message`: The message to broadcast.
async fn broadcast_message(clients: SharedClients, message: &str) {
    let mut clients_to_remove = Vec::new();
    {
        let mut clients = clients.lock().await;
        for (index, writer) in clients.iter_mut().enumerate() {
            if writer
                .write_all(format!("{}\n", message).as_bytes())
                .await
                .is_err()
            {
                clients_to_remove.push(index);
            }
        }
    }

    // Remove disconnected clients
    let mut clients = clients.lock().await;
    for &index in clients_to_remove.iter().rev() {
        clients.remove(index);
    }
}

/// Tests for the server module.
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpStream;

    #[tokio::test]
    async fn test_parse_private_message() {
        // Valid private message
        let input = "/msg 2 Hello, Client 2!";
        let result = parse_private_message(input);
        assert_eq!(result, Some((2, "Hello, Client 2!")));

        // Invalid private message (missing client ID)
        let invalid_input = "/msg Hello, Client!";
        let invalid_result = parse_private_message(invalid_input);
        assert_eq!(invalid_result, None);

        // Invalid private message (missing command prefix)
        let invalid_input = "msg 2 Hello!";
        let invalid_result = parse_private_message(invalid_input);
        assert_eq!(invalid_result, None);
    }

    #[tokio::test]
    async fn test_send_private_message() {
        let clients = SharedClients::default();

        // Create a listener to simulate the server
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn a simulated client
        let client = tokio::spawn(async move {
            let stream = tokio::net::TcpStream::connect(addr).await.unwrap();
            let mut buf_reader = BufReader::new(stream);
            let mut received_message = String::new();

            // Read the private message
            buf_reader.read_line(&mut received_message).await.unwrap();
            received_message
        });

        // Accept the simulated client connection on the server
        let (socket, _) = listener.accept().await.unwrap();
        let (_reader, writer) = socket.into_split();

        // Add the writer to the clients list
        clients.lock().await.push(writer);

        // Test sending a private message
        let message = "[Private] Client 1: Hello!";
        send_private_message(clients.clone(), 1, message).await;

        // Assert that the client received the correct private message
        let received_message = client.await.unwrap();
        assert_eq!(received_message.trim(), message);
    }

    #[tokio::test]
    async fn test_broadcast_message() {
        let clients = SharedClients::default();

        // Create a listener for the server
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        // Spawn two simulated clients
        let client1 = tokio::spawn(async move {
            let stream = TcpStream::connect(addr).await.unwrap();
            let mut buf_reader = BufReader::new(stream);
            let mut received_message = String::new();

            buf_reader.read_line(&mut received_message).await.unwrap();
            received_message
        });

        let client2 = tokio::spawn(async move {
            let stream = TcpStream::connect(addr).await.unwrap();
            let mut buf_reader = BufReader::new(stream);
            let mut received_message = String::new();

            buf_reader.read_line(&mut received_message).await.unwrap();
            received_message
        });

        // Accept two client connections and add their writers to the shared list
        for _ in 0..2 {
            let (socket, _) = listener.accept().await.unwrap();
            let (_reader, writer) = socket.into_split();
            clients.lock().await.push(writer);
        }

        // Broadcast a message
        let message = "Hello, everyone!";
        broadcast_message(clients.clone(), message).await;

        // Assert that both clients received the broadcast message
        let response1 = client1.await.unwrap();
        let response2 = client2.await.unwrap();
        assert_eq!(response1.trim(), message);
        assert_eq!(response2.trim(), message);
    }
}
