use clap::{Arg, Parser, Subcommand};
use smse::Smse;
use std::process::ExitCode;
use std::rc::Rc;
use unitn_market_2022::market::Market;
use SGX::market::sgx::SGX;
use TASE::TASE;
use ZSE::market::ZSE;

#[derive(Debug, Parser)]
#[clap(about, author, version)]
pub struct Args {
    /// Name of the strategy the trader is supposed to use.
    pub strategy: String,
    /// List of markets the trader should work with.
    pub markets: Vec<String>,
    /// Verbose level.
    #[arg(short, long, default_value_t = false)]
    pub verbose: bool,
    /// Log level.
    #[arg(short, long, default_value = "error")]
    pub log_level: String,
}

struct MarketFactory {
    allowed_markets: Vec<String>,
}

impl MarketFactory {
    fn gen_market(market_name: &String) -> Option<Rc<dyn Market>> {
        let market_name = market_name.clone().to_ascii_lowercase();
        None
    }
}

fn parse_markets(markets: &[String]) {
    for market_name in markets.iter() {
        println!("WE HAVE {}", market_name);
    }
}

fn main() {
    let args = Args::parse();

    if args.markets.is_empty() {
        // we need at least one market to work with
        println!("At least one market is required");
        std::process::exit(1);
    }

    parse_markets(&args.markets);
}
