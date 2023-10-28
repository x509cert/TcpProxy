use getopts::Options;
use std::env;

#[derive(Debug)]
#[derive(PartialEq, Eq, Clone, Copy)]
pub enum FuzzDirection {
    ClientToServer,
    ServerToClient,
    Both,
    None,
}

pub struct ParsedArgs {
    pub proxy: String,
    pub server: String,
    pub direction: FuzzDirection,      
    pub aggressiveness: u32     
}

pub fn parse_args() -> ParsedArgs {
    let args: Vec<String> = env::args().collect();
    let mut opts = Options::new();

    opts.optopt("p", "proxy", "Set proxy IP address and port (format: IP:PORT)", "IP:PORT");
    opts.optopt("s", "server", "Set server IP address and port (format: IP:PORT)", "IP:PORT");
    opts.optopt("d", "direction", "Set direction (b (bi-di), c (server->client), or s (client->server))", "DIRECTION");
    opts.optopt("a", "aggressiveness", "Set fuzzing aggressiveness (1-100)", "VALUE");

    let matches = opts.parse(&args[1..]).expect("Failed to parse arguments");

    let proxy = matches.opt_str("p").expect("Proxy not provided");
    let server = matches.opt_str("s").expect("Server not provided");

    let direction = match matches.opt_str("d").unwrap_or("n".to_string()).as_str() {
        "s" => FuzzDirection::ClientToServer,
        "c" => FuzzDirection::ServerToClient,
        "b" => FuzzDirection::Both,
        _ => FuzzDirection::None,
    };

    let aggressiveness_str = matches.opt_str("a").unwrap_or("25".to_string());
    let aggressiveness: u32 = aggressiveness_str.parse::<u32>().expect("Aggressiveness must be a number between 0-100");
    if !(0..=100).contains(&aggressiveness) {
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