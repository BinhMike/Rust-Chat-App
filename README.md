[![Review Assignment Due Date](https://classroom.github.com/assets/deadline-readme-button-22041afd0340ce965d47ae6ef1cefeee28c7c493a6346c4f15d667ab976d596c.svg)](https://classroom.github.com/a/c2Cd-Xpe)
# Rust ChatApp

## Description
Rust ChatApp is a simple, asynchronous chat application built in Rust. The primary goal of this project is to build network applications in Rust, utilizing asynchronous I/O with tokio. The project implements a client-server architecture where multiple clients can connect to the server and communicate in real-time. Each message is either broadcast to all connected clients or sent as a private message to a specific client.

### Key Features
- Broadcast Messaging: Clients can send messages to all connected users.
- Private Messaging: Clients can send direct messages to specific users using the `/msg <client_id> <message>` command.
- Client IDs: Each client is assigned a unique ID when they connect to the server, which is used for private messaging.
- Graceful Disconnection: The server handles client disconnections smoothly, ensuring that remaining clients continue to operate normally.
- Self-Identification: Clients' own messages are tagged with `(Me)` for better clarity.
- Concurrency: The server can handle multiple client connections concurrently using asynchronous tasks.

### Key Third-Party Crates
- [tokio](https://crates.io/crates/tokio): Provides the async runtime for handling asynchronous tasks and I/O operations.
- [tokio-stream](https://crates.io/crates/tokio-stream): Manages asynchronous streams, used to handle connections.
- [futures](https://crates.io/crates/futures): Offers utilities for working with asynchronous code.

---

## Installation
To install and run Rust ChatApp, follow these steps:

### 1. Clone the repository:
git clone git@github.com:rustvu-2024f/project-BinhMike.git
cd project-BinhMike

### 2. Build the project:
Compile the project using Cargo, Rust's package manager:
cargo build

### 3. Run the server:
Start the server on a specific address (e.g., `127.0.0.1:808`):
cargo run -- server 0.0.0.0:8080

### 4. Run the client:
Connect a client to the server using the following command:
cargo run -- client 127.0.0.1:8080

### 5. Simulate multiple clients:
Run multiple clients in separate terminals to simulate a multi-user chat environment.

---

## How to Use
### Server Setup
1. Start the server: Use the following command to start the server:
   cargo run -- server 0.0.0.0:8080
   By default, the server listens for connections on `127.0.0.1:808`.

2. Server Output: The server logs activity to the console, including:
   - New client connections.
   - Messages sent by clients.
   - Private messages sent between clients.
   - Client disconnections.

### Client Setup
1. Connect a client: Use the following command to connect a client to the server:
   cargo run -- client 127.0.0.1:8080

2. Send and receive messages:
   - Type a message in the client terminal and press Enter. The message will be sent to the server and broadcast to all connected clients.
   - Messages from other clients will appear in your terminal. Your own messages are tagged with `(Me)`.

3. Send private messages:
   - Use the `/msg <client_id> <message>` command to send a private message to a specific client. For example:
     /msg 2 Hello, Client 2!

4. Run multiple clients:
   - Open multiple terminals and run the client command in each. This allows you to simulate a multi-user chat environment where clients can send broadcast and private messages.

### Run Unit Tests and Integration Tests
1. Run unit tests:
   - Inside server.rs, click on Run Tests. There are 3 unit tests in total

2. Run integraion tests:
   - cargo test --test integration_test

### Documentation
1. Generate the documentation:
   - cargo doc
2. Open the documentation in your browser:
   - cargo doc --open
This automatically opens the main page of the documentation for your project in your default browser.

---

## Example Usage
### Starting the Server:
cargo run -- server 0.0.0.0:8080
Output:
Server listening on 127.0.0.1:808
New connection: 127.0.0.1:58914 (Client 1)
New connection: 127.0.0.1:58915 (Client 2)

### Starting a Client:
cargo run -- client 127.0.0.1:8080
Output:
Connected as Client 1
Hello, everyone!

### Private Messaging:
- Client 1 sends a private message to Client 2:
  /msg 2 Hello, Client 2!

- Output on Client 2's terminal:
  [Private] Client 1: Hello, Client 2!

### Broadcasting:
- Client 1 broadcasts a message:
  Hello, everyone!
- Output on all clients:
  Client 1: Hello, everyone!

---

## Future Improvements
- Chat Rooms: Allow clients to join specific rooms for isolated conversations.
- Message History: Enable clients to view past messages when they connect.
- Structured Messages: Use `serde` to handle structured messages in JSON format.

---