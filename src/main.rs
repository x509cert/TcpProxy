/*
A simple TCP proxy fuzzer, just for the fun of it
Michael Howard (mikehow@microsoft.com)
Azure Data Platform Security 
*/

use tokio::{io::{self, AsyncReadExt, AsyncWriteExt}, 
    net::{TcpListener, TcpStream}, select};

mod fuzz;
mod parseargs;
mod naughty;

use parseargs::FuzzDirection;
use naughty::read_naughty_words;

const BUFF_SIZE: usize = 4096;

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

    println!("\nRusty Proxy Fuzzer {}\nWritten by Michael Howard\nAzure Data Platform. Microsoft Corp.", env!("CARGO_PKG_VERSION"));

    // get naughty strings from file system
    let mut file_paths: Vec<&str> = vec![];
    if !args.naughty.is_empty()  {
        if args.naughty.contains('h') { file_paths.push("html5.txt");}
        if args.naughty.contains('j') { file_paths.push("json.txt");}
        if args.naughty.contains('n') { file_paths.push("naughty.txt");}
        if args.naughty.contains('s') { file_paths.push("sql.txt");}
        if args.naughty.contains('x') { file_paths.push("xml.txt");}
        if args.naughty.contains('y') { file_paths.push("xss.txt");}
        if args.naughty.contains('z') { file_paths.push("xxe.txt");}
    }

    let naughty_words = read_naughty_words(file_paths.clone()).await?;
    if !naughty_words.is_empty() {
        println!("All 'naughty' files read, {} lines", naughty_words.len());
    }

    println!("Proxying {} -> {}", client, server);
    println!("Fuzzing direction is {:?} with aggressiveness {}%", direction, aggressiveness);

    loop {
        let mut buf: [u8; BUFF_SIZE] = [0; BUFF_SIZE];

        let (client, _) = listener.accept().await?;
        let server = TcpStream::connect(server).await?;

        let (mut cread, mut cwrite) = client.into_split();
        let (mut sread, mut swrite) = server.into_split();

        let task_c2s = tokio::spawn(async move {
            loop {
                match cread.read(&mut buf).await {
                    Ok(0) => return,
                    Ok(n) => {
                        if (direction == FuzzDirection::ClientToServer || direction == FuzzDirection::Both) && aggressiveness > 0 {
                            let _res = fuzz::fuzz_buffer(&mut buf, &naughty_words, aggressiveness);
                        } 
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
                        let mut bufferlen = n;
                        if (direction == FuzzDirection::ServerToClient || direction == FuzzDirection::Both) && aggressiveness > 0 {
                            let result =  fuzz::fuzz_buffer(&mut buf, &naughty_words, aggressiveness);
                            if let Some(v) = result {
                                bufferlen = v;
                            }  
                        }
                        cwrite.write_all(&buf[..bufferlen]).await.expect("Failed to write to client")
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

