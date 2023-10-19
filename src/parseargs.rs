use getopts::Options;
use std::env;

pub struct ParsedArgs {
    pub proxy: String,
    pub server: String,
    pub direction: String,      // presently unused
    pub aggressiveness: u8      // same
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