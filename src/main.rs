/*
A simple TCP proxy fuzzer, just for the fun of it
Michael Howard (mikehow@microsoft.com)
Azure Datam Platform Security 

Future work
    Hook up:
    - fuzzing direction
    - aggressiveness
*/

use getopts::Options;
use std::env;
use tokio::{io::{self, AsyncReadExt, AsyncWriteExt}, 
    net::{TcpListener, TcpStream}, select};

struct ParsedArgs {
    proxy: String,
    server: String,
    direction: String,      // presently unused
    aggressiveness: u8      // same
}

mod fuzz;

#[tokio::main]
async fn main() -> io::Result<()> {

    // example cmd-line: -p 127.0.0.1:8080 -s 192.168.1.20:9000 -d b -a 75
    let args = parse_args();

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

fn parse_args() -> ParsedArgs {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();

    opts.optopt("p", "proxy", "Set proxy IP address and port (format: IP:PORT)", "IP:PORT");
    opts.optopt("s", "server", "Set server IP address and port (format: IP:PORT)", "IP:PORT");
    opts.optopt("d", "direction", "Set direction (b (bi-di), c (server->client), or s (client->server))", "DIRECTION");
    opts.optopt("a", "aggressiveness", "Set fuzzing aggressiveness (1-100)", "VALUE");

    let matches = opts.parse(&args[1..]).expect("Failed to parse arguments");

    let proxy = matches.opt_str("p").expect("Proxy not provided");
    let server = matches.opt_str("s").expect("Server not provided");

    let direction = matches.opt_str("d").expect("Direction not provided");
    if !["b", "c", "s"].contains(&direction.as_str()) {
        panic!("Invalid direction value!");
    }

    let aggressiveness_str = matches.opt_str("a").unwrap_or("33".to_string());
    let aggressiveness = aggressiveness_str.parse::<u8>().expect("Aggressiveness must be a number");
    if aggressiveness < 1 || aggressiveness > 100 {
        panic!("Aggressiveness value out of range!");
    }

    // return the args
    ParsedArgs {
        proxy,
        server,
        direction,
        aggressiveness
    }
}