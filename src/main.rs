/*
A simple TCP proxy fuzzer, just for the fun of it
Michael Howard (mikehow@microsoft.com)
Azure Datam Platform Security 

Future work
    Hook up:
    - fuzzing direction
    - aggressiveness
*/

use tokio::{io::{self, AsyncReadExt, AsyncWriteExt}, 
    net::{TcpListener, TcpStream}, select};

mod fuzz;
mod parseargs;

#[tokio::main]
async fn main() -> io::Result<()> {

    // example cmd-line: -p 127.0.0.1:8080 -s 192.168.1.20:9000 -d b -a 75
    let args = parseargs::parse_args();

    let client: &str = &args.proxy;
    let server: &str = &args.server;
    
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
                        // Fuzz server to client traffic, don't care about the return
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

