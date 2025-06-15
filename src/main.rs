use tokio::io::AsyncWriteExt;
use tokio::net::TcpListener;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Logs from your program will appear here!");


    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    println!("Listening on: 127.0.0.1:6379");


    loop {
        let (mut socket, addr) = listener.accept().await?;
        println!("Accepted connection from: {}", addr);

        // Spawn a new task for each connection
        tokio::spawn(async move {
            // Send PONG response
            if let Err(e) = socket.write_all(b"+PONG\r\n").await {
                println!("Error writing to socket: {}", e);
            }
        });
    }
}