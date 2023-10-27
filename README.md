# Rust TcpProxyFuzzer
A simple multi-threaded TCP Proxy Fuzzer written in Rust.

Usage is:


TcpProxy -s <server_ip:port> -p <proxy_ip:port> -d <b|c|s|n> [-a 1-100]

Where:

  -s Is the server IP address and port this proxy is to connect to. For example 192.168.1.11:8080
  
  -p This proxy IP address and port to listen on. For example, 127.0.0.1:8088
  
  -d The fuzzing direction. b == bi-directional, c == fuzz the client (ie; server->client), s == fuzz the server (ie; client->server) and n == none
  
  -a Is the fuzzing aggressiveness, between 0-100, default is 33. This is the only optional arg
