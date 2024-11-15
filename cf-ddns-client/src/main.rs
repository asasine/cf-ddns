use std::process::ExitCode;

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
    #[arg(long)]
    token: String,
}

#[derive(clap::Args)]
#[group(required = true, multiple = false)]
struct ZoneArgs {
    /// The name of the Cloudflare DNS zone.
    #[arg(long, group = "zone")]
    zone_name: Option<String>,

    /// The ID of the Cloudflare DNS zone. This is an alternative to `zone_name`.
    #[arg(long, group = "zone")]
    zone_id: Option<String>,
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

    println!("IP: {}", ip);

    let client = Cloudflare::try_new(&args.token).unwrap();
    let zone_id = match (args.zone.zone_name, args.zone.zone_id) {
        (Some(zone_name), _) => client.get_zone_id(&zone_name).unwrap(),
        (_, Some(zone_id)) => zone_id,
        _ => unreachable!("Either zone_name or zone_id must be provided."),
    };

    println!("zone id: {zone_id}");

    ExitCode::SUCCESS
}
