// A simple TCP proxy, just for the fun of it
// Michael Howard (mikehow@microsoft.com)
// Azure Database Security 
// Oct, 12th 2023 - Initial
use std::io::{self, Read, Write};
use std::net::{SocketAddr, TcpListener, TcpStream};
use std::thread;

fn handle_client( client: TcpStream, server_addr: SocketAddr) {
    match TcpStream::connect(server_addr) {
        Ok(server) => {
            let client_clone = client.try_clone().unwrap();
            let server_clone = server.try_clone().unwrap();

            let _ = thread::spawn(move || transfer_data(&client_clone, &server_clone));
            let _ = thread::spawn(move || transfer_data(&server, &client));
        }
        Err(e) => {
            println!("Failed to connect to server: {}", e);
        }
    }
}

fn transfer_data(mut source: &TcpStream, mut destination: &TcpStream) {
    let mut buf = [0; 4096];
    loop {
        match source.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                println!("data");
                let _ = destination.write_all(&buf[..n]);
            }
            Err(_) => break,
        }
    }
}

fn main() -> io::Result<()> {
    let local_addr: SocketAddr = "127.0.0.1:1533".parse().unwrap();
    let server_addr: SocketAddr = "127.0.0.1:1434".parse().unwrap();

    let listener = TcpListener::bind(local_addr)?;

    println!("Proxying from {} to {}", local_addr, server_addr);

    for stream in listener.incoming() {
        match stream {
            Ok(client) => {
                println!("1");
                thread::spawn(move || handle_client(client, server_addr));
            }
            Err(e) => {
                println!("Failed to accept client: {}", e);
            }
        }
    }

    Ok(())
}
