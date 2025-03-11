//! A simple P2P chat application with a server and multiple clients.
//! The server broadcasts messages to all connected clients, and each client displays messages
//! from others, tagging its own messages with "(Me)".

mod client;
mod server;

use std::env;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        eprintln!("Usage: {} [server|client] [address]", args[0]);
        return;
    }

    let mode = &args[1];
    match mode.as_str() {
        "server" => {
            let address = args
                .get(2)
                .map(String::from)
                .unwrap_or_else(|| "0.0.0.0:8080".to_string());
            server::run_server(&address).await.unwrap();
        }
        "client" => {
            let address = args
                .get(2)
                .map(String::from)
                .unwrap_or_else(|| "127.0.0.1:8080".to_string());
            client::run_client(&address).await.unwrap();
        }
        _ => eprintln!("Unknown mode: {}. Use 'server' or 'client'.", mode),
    }
}
