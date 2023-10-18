// A simple TCP proxy, just for the fun of it
// Michael Howard (mikehow@microsoft.com)
// Azure Database Security 
// Oct, 12th 2023 - Initial

use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;

mod fuzz;

#[tokio::main]
async fn main() -> io::Result<()> {
    let client = "127.0.0.1:1533";
    let server = "127.0.0.1:1433";
    
    let listener = TcpListener::bind(client).await?;

    println!("Proxying {} -> {}", client, server);

    loop {
        let mut buf: [u8; 4096] = [0; 4096];

        let (client, _) = listener.accept().await?;
        let server = TcpStream::connect(server).await?;

        let (mut eread, mut ewrite) = client.into_split();
        let (mut oread, mut owrite) = server.into_split();

        let task_c2s = tokio::spawn(async move {
            loop {
                match eread.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(n) => {
                        // Modify buf here if needed (from client to server)    
                        // TODO                  
                        owrite.write_all(&buf[..n]).await.expect("Failed to write to server");
                    }
                    Err(_) => return,
                }
            }
        });

        let task_s2c = tokio::spawn(async move {
            loop {
                match oread.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(n) => {
                        // Fuzz server to client traffic
                        _ = fuzz::fuzz_buffer(&mut buf, 10);
                        ewrite.write_all(&buf[..n]).await.expect("Failed to write to client");
                    }
                    Err(_) => return,
                }
            }
        });

        // Tokio's select macro to wait for the results
        select! {
            _ = task_c2s => {},
            _ = task_s2c => {},
        }
    }
}
