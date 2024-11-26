use std::process::ExitCode;
use std::path::PathBuf;

use clap::Parser;

use cf_ddns_client::cloudflare::Cloudflare;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    /// The URL to send a GET request to.
    #[arg(long, default_value = "https://cf-ddns.adam-sasine.workers.dev")]
    url: String,

    /// The Cloudflare Zone to update.
    #[command(flatten)]
    zone: ZoneArgs,

    /// The API token to authenticate with the Cloudflare API.
    #[command(flatten)]
    token: TokenArgs,

    /// The DNS record to update.
    #[command(flatten)]
    record: RecordArgs,

    /// Include debug output.
    #[arg(long)]
    debug: bool,
}

#[derive(clap::Args)]
#[group(required = true, multiple = false)]
struct TokenArgs {
    /// The API token to authenticate with the Cloudflare API.
    #[arg(long)]
    token: Option<String>,

    /// The path to a file containing the API token to authenticate with the Cloudflare API.
    #[arg(long)]
    token_file: Option<PathBuf>,
}

impl TokenArgs {
    fn token(&self) -> String {
        let token = match (&self.token, &self.token_file) {
            (Some(token), None) => token.to_string(),
            (None, Some(token_file)) => {
                std::fs::read_to_string(token_file).expect("Could not read token file")
            }
            _ => unreachable!("Clap should ensure either token or token_file is provided."),
        };

        token.trim().to_string()
    }
}

#[derive(clap::Args)]
struct ZoneArgs {
    /// The name of the Cloudflare DNS zone.
    #[arg(long)]
    zone_name: String,

    /// The ID of the Cloudflare DNS zone, if known,
    ///
    /// This can be provided to avoid the need to look up the zone ID by name. It should match the ID of [`Self::zone_name`].
    #[arg(long)]
    zone_id: Option<String>,
}

#[derive(clap::Args)]
#[group(required = true, multiple = false)]
struct RecordArgs {
    /// The name of the record to update.
    #[arg(long)]
    record_name: Option<String>,

    /// The ID of the record to update, instead of looking up from the name.
    #[arg(long)]
    record_id: Option<String>,
}

fn main() -> ExitCode {
    let args = Args::parse();

    let ip = match cf_ddns_client::worker::get_ip(&args.url) {
        Ok(ip) => ip,
        Err(err) => {
            eprintln!("{}", err);
            return ExitCode::FAILURE;
        }
    };

    if args.debug {
        eprintln!("IP: {}", ip);
    }

    let client = Cloudflare::try_new(&args.token.token()).unwrap();
    let zone_id = match args.zone.zone_id {
        Some(zone_id) => zone_id,
        None => match client.get_zone_id(&args.zone.zone_name) {
            Ok(zone_id) => zone_id,
            Err(err) => {
                eprintln!("Could not get zone ID: {}", err);
                return ExitCode::FAILURE;
            }
        },
    };

    if args.debug {
        eprintln!("zone id: {zone_id}");
    }

    let record_id = match (args.record.record_name, args.record.record_id) {
        (Some(record_name), _) => {
            let full_record_name = format!("{}.{}", record_name, args.zone.zone_name);
            match client.get_record_id(&zone_id, &full_record_name) {
                Ok(record_id) => record_id,
                Err(err) => {
                    eprintln!("Could not get record ID: {}", err);
                    return ExitCode::FAILURE;
                }
            }
        }
        (_, Some(record_id)) => record_id,
        _ => unreachable!("Clap should ensure either record_name or record_id is provided."),
    };

    if args.debug {
        eprintln!("record id: {record_id}");
    }

    let record = match client.update_record(&zone_id, &record_id, ip) {
        Ok(record) => record,
        Err(err) => {
            eprintln!("Could not update record: {}", err);
            return ExitCode::FAILURE;
        }
    };

    println!("{} => {ip}", record.name);
    ExitCode::SUCCESS
}
