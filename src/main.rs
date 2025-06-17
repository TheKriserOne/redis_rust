mod parser;

use std::sync::{Arc, Weak};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::join;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_stream::{wrappers, StreamExt};
use crate::parser::RESPtypes;

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
                        let mut buffer= vec![0u8; 1024];; // Buffer to read data
                        while let Ok(n) = stream.read(&mut buffer).await {

                            let received = String::from_utf8(Vec::from(&buffer[..n])).unwrap();
                            let parsed = RESPtypes::from_string(&received);
                            match parsed {
                                RESPtypes::Arrays(val) => {
                                    // if let Some(RESPtypes::BulkStrings(s)) = &val[0] {
                                    //
                                    // }

                                    let mut val = val.iter();
                                    if let Some(RESPtypes::BulkStrings(s)) = val.next() {
                                        if s == "ECHO" {
                                            let echoed_val = val.fold("".to_string(), |initial, current| {
                                                initial + &*current.to_resp_string()
                                            });
                                            stream
                                                .write_all(echoed_val.as_ref())
                                                .await
                                                .expect("Failed to write PONG response");
                                        }
                                        else if s == "PING" {
                                            stream
                                                .write_all(b"+PONG\r\n")
                                                .await
                                                .expect("Failed to write PONG response");
                                        }
                                    }
                                }
                                _ => {
                                    stream.write_all(b"-ERR Protocol error: invalid RESP format\r\n").await.unwrap();
                                }
                            }
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
