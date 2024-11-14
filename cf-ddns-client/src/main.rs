use std::net::IpAddr;
use std::process::ExitCode;

use clap::Parser;
use reqwest::blocking::get;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// The URL to send a GET request to.
    #[arg(default_value = "https://cf-ddns.adam-sasine.workers.dev")]
    url: String,
}

fn get_ip(url: &str) -> Result<IpAddr, &str> {
    let response = get(url)
        .map_err(|_| "Failed to send request.")?
        .text()
        .map_err(|_| "Failed to get response body.")?;

    response
        .parse::<IpAddr>()
        .map_err(|_| "Failed to parse IP address.")
}

fn main() -> ExitCode {
    let args = Args::parse();
    let ip = match get_ip(&args.url) {
        Ok(ip) => ip,
        Err(err) => {
            eprintln!("{}", err);
            return ExitCode::FAILURE;
        }
    };

    println!("{}", ip);
    ExitCode::SUCCESS
}
