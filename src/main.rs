use std::sync::{Arc, Weak};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::join;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_stream::{wrappers, StreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:6379").await?;
    let stream_of_conn = TcpListenerStream::new(listener);
    let handles = Arc::new(Mutex::new(Vec::new()));
    join!(
        listen(stream_of_conn, handles.clone()),
        stream_connection(handles.clone())
    );
    Ok(())

    // Spawn a new task for each connection
    // tokio::spawn(async move {
    //     // Send PONG response
    //     if let Err(e) = socket.write_all(b"+PONG\r\n").await {
    //         println!("Error writing to socket: {}", e);
    //     }
    // });
}

async fn listen(
    mut listener: TcpListenerStream,
    handles: Arc<Mutex<Vec<TcpStream>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    while let Some(Ok(mut stream)) = listener.next().await {
        let mut handles = handles.lock().await;
        handles.push(stream);
    }
    Ok(())
}
async fn stream_connection(handles: Arc<Mutex<Vec<TcpStream>>>) {
    loop {
        let mut streams_to_process = {
            let mut streams = handles.lock().await;
            if streams.is_empty() {
                continue;
            }
            std::mem::take(&mut *streams) // Take all streams, leaving Vec empty
        };

        let _: Vec<_> = streams_to_process
            .into_iter()
            .map(|mut stream| {
                tokio::spawn(async move {
                    loop {
                        let mut buffer = Vec::with_capacity(1024);
                        while let Ok(n) = stream.read(&mut buffer).await {

                            stream
                                .write_all(b"+PONG\r\n")
                                .await
                                .expect("Failed to write PONG response");
                            let received = String::from_utf8_lossy(&buffer[..n]);
                            println!("Received: {}", received);
                            buffer.clear(); // Clear the buffer for the next read
                        }
                    }
                });
            })
            .collect();
    }
}
// async fn send_pong(socket: &mut TcpStream) -> JoinHandle<()> {
//     tokio::spawn(async move {
//         if let Err(e) = socket.write_all(b"+PONG\r\n").await {
//             println!("Error writing to socket: {}", e);
//         }
//     })
// }
