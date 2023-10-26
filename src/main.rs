/*
A simple TCP proxy fuzzer, just for the fun of it
Michael Howard (mikehow@microsoft.com)
Azure Data Platform Security 

*/

use tokio::{io::{self, AsyncReadExt, AsyncWriteExt}, 
    net::{TcpListener, TcpStream}, select};

mod fuzz;
mod parseargs;
use parseargs::FuzzDirection;

#[tokio::main]
async fn main() -> io::Result<()> {

    // example cmd-line: 
    // -p 127.0.0.1:8080 -s 192.168.1.20:9000 -d b -a 75
    // means: 
    // listen on 127.0.0.1 port 8080
    // proxy the traffic to 192.168.1.20 port 9000. 
    // fuzz in both directions 'b' == both
    // aggressiveness of 75%, so 75% of traqffic is fuzzed
    let args = parseargs::parse_args();

    let client: &str = &args.proxy;
    let server: &str = &args.server;
    let direction = args.direction;
    let aggressiveness: u32 = args.aggressiveness;
    
    let listener = TcpListener::bind(client).await?;

    println!("Rusty Proxy Fuzzer {}\nWritten by Michael Howard\nAzure Data Platform. Microsoft Corp.", env!("CARGO_PKG_VERSION"));
    println!("Proxying {} -> {}", client, server);
    println!("Fuzzing direction is {:?} with aggressiveness {}", direction, aggressiveness);

    loop {
        let mut buf: [u8; 4096] = [0; 4096];

        let (client, _) = listener.accept().await?;
        let server = TcpStream::connect(server).await?;

        let (mut cread, mut cwrite) = client.into_split();
        let (mut sread, mut swrite) = server.into_split();

        let task_c2s = tokio::spawn(async move {
            loop {
                match cread.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(n) => {
                        // Modify buf here if needed (from client to server)    
                        // TODO                  
                        swrite.write_all(&buf[..n]).await.expect("Failed to write to server");
                    }
                    Err(_) => return,
                }
            }
        });

        let task_s2c = tokio::spawn(async move {
            loop {
                match sread.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(n) => {
                        // 'c' == Fuzz server -> client traffic, 'b' == both drections
                        // don't care about the return
                        if (direction == FuzzDirection::ServerToClient || direction == FuzzDirection::Both) && aggressiveness > 0 {
                            let _res = fuzz::fuzz_buffer(&mut buf, aggressiveness);
                        }
                        cwrite.write_all(&buf[..n]).await.expect("Failed to write to client")
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

