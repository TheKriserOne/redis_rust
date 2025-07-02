mod parser;

use std::collections::HashMap;
use crate::parser::RESPtypes;
use std::sync::{Arc, Weak};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::join;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio_stream::wrappers::TcpListenerStream;
use tokio_stream::{wrappers, StreamExt};
use crate::parser::RESPtypes::{Arrays, BulkStrings};

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

}

async fn listen(
    mut listener: TcpListenerStream,
    handles: Arc<Mutex<Vec<TcpStream>>>,
) -> Result<(), Box<dyn std::error::Error>> {
    while let Some(Ok(stream)) = listener.next().await {
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
                    let mut data: HashMap<String, String> = HashMap::new();
                    loop {

                        let mut buffer = vec![0u8; 1024]; // Buffer to read data
                        while let Ok(n) = stream.read(&mut buffer).await {

                            let received = String::from_utf8(Vec::from(&buffer[..n])).unwrap();
                            let parsed = RESPtypes::from_string(&received);

                            match parsed {
                                Arrays(val) => {

                                    let mut val = val.iter();
                                    if let Some(BulkStrings(s)) = val.next() {

                                        if s == "ECHO" {
                                            stream
                                                .write_all(
                                                    val.next().unwrap().to_resp_string().as_ref(),
                                                )
                                                .await
                                                .expect("Failed to write PONG response");

                                        } else if s == "PING" {
                                            stream
                                                .write_all(b"+PONG\r\n")
                                                .await
                                                .expect("Failed to write PONG response");
                                        }
                                        else if s == "SET" {
                                            data.insert(val.next().unwrap().get(), val.next().unwrap().get());
                                            stream.write_all(b"+OK\r\n").await.expect("TODO: panic message");
                                        }
                                        else if s == "GET" {
                                            if let Some(BulkStrings(query)) = val.next() {

                                                match data.get(query) {
                                                    Some(val) => {
                                                        stream.write_all(BulkStrings(val.clone()).to_resp_string().as_ref()).await.expect("TODO: panic message");
                                                    }
                                                    None => {
                                                        stream.write_all(b"$-1\r\n").await.expect("TODO: panic message");
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                                _ => {
                                    stream
                                        .write_all(b"-ERR Protocol error: invalid RESP format\r\n")
                                        .await
                                        .unwrap();
                                }
                            }
                        }
                    }
                });
            })
            .collect();
    }
}

