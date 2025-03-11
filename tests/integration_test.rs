use std::process::{Child, Command};
use std::time::Duration;
use tokio::time::sleep;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
    net::TcpStream,
};

/// Starts the server as a background process.
async fn start_server() -> Child {
    Command::new("cargo")
        .arg("run")
        .arg("--")
        .arg("server")
        .arg("127.0.0.1:8080")
        .spawn()
        .expect("Failed to start server")
}

#[tokio::test]
async fn test_broadcast_and_private_message() {
    // Start the server in the background
    let mut server_process = start_server().await;
    sleep(Duration::from_secs(2)).await; // Allow time for the server to start

    // Connect the first client
    let stream_1 = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    let (reader_1, mut writer_1) = stream_1.into_split();
    let mut buf_reader_1 = BufReader::new(reader_1);

    // Read and verify Client 1's ID
    let mut id_line_1 = String::new();
    buf_reader_1.read_line(&mut id_line_1).await.unwrap();
    assert!(id_line_1.starts_with("Your ID: 1"));

    // Connect the second client
    let stream_2 = TcpStream::connect("127.0.0.1:8080").await.unwrap();
    let (reader_2, _writer_2) = stream_2.into_split();
    let mut buf_reader_2 = BufReader::new(reader_2);

    // Read and verify Client 2's ID
    let mut id_line_2 = String::new();
    buf_reader_2.read_line(&mut id_line_2).await.unwrap();
    assert!(id_line_2.starts_with("Your ID: 2"));

    // Test broadcasting: Client 1 sends a message to all clients
    writer_1.write_all(b"Hello from Client 1\n").await.unwrap();

    // Verify the message is received by Client 2
    let mut received_message = String::new();
    buf_reader_2.read_line(&mut received_message).await.unwrap();
    assert!(received_message.contains("Client 1: Hello from Client 1"));

    // Test private messaging: Client 1 sends a private message to Client 2
    writer_1
        .write_all(b"/msg 2 Hello, Client 2!\n")
        .await
        .unwrap();

    // Verify the private message is received by Client 2
    let mut private_message = String::new();
    buf_reader_2.read_line(&mut private_message).await.unwrap();
    assert!(private_message.contains("[Private] Client 1: Hello, Client 2!"));

    // Shut down the server
    server_process.kill().unwrap();
}
