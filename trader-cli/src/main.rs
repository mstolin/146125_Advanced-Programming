//! This is a CLI tool to execute a trader using the *trader* library.
//!
//! # Installation
//!
//! From the workspace directory execute the following:
//!
//! ```shell
//! $ cargo install --path ./trader-cli
//! ```
//!
//! After that, you can use the command as `$ trader-cli`.
//!
//! # Usage
//!
//! To see its features, execute `$ trader-cli --help`.
//!
//! # Examples
//!
//! *Run `AverageSeller` for 30 days, every 60 minutes on SGX and TASE and print history as JSON*
//! ```shell
//! $ trader-cli average-seller sgx tase -d 30 -m 60 --as-json
//! ```
//!
//! *Run `AverageSeller` for 7 days, every 10 minutes on SGX, SMSE, and TASE wth 30.000.00 EUR start capital, and print history
//! as plain text*
//! ```shell
//! $ trader-cli average-seller sgx smse tase -d 7 -m 10 -c 3000000
//! ```

use chrono::Local;
use clap::Parser;
use env_logger::Env;
use smse::Smse;
use std::cell::RefCell;
use std::fs::File;
use std::io;
use std::io::Write;
use std::path::PathBuf;
use std::rc::Rc;
use trader::trader::{StrategyIdentifier, Trader};
use unitn_market_2022::market::Market;
use SGX::market::sgx::SGX;
use TASE::TASE;
use ZSE::market::ZSE;

/// Represents a market
type MarketRef = Rc<RefCell<dyn Market>>;

/// Possible arguments for the executable.
#[derive(Debug, Parser)]
#[clap(about, author, version)]
pub struct Args {
    /// Name of the strategy the trader is supposed to use.
    /// Available strategy names: average-seller, stingy.
    pub strategy: String,
    /// List of markets the trader should work with.
    /// Available market names: sgx, smse, tase, zse.
    pub markets: Vec<String>,
    /// The starting capital in EUR for the trader.
    #[arg(short, long, default_value_t = 1_000_000.0)]
    pub capital: f32,
    /// The number of days this trader is suppose to run.
    #[arg(short, long, default_value_t = 1)]
    pub days: u32,
    /// The log level of the application.
    #[arg(short, long, default_value = "error")]
    pub log_level: String,
    /// The interval of minutes, when the trader applies its strategy
    /// during the day.
    #[arg(short, long, default_value_t = 60)]
    pub minute_interval: u32,
    /// Indicates if the history should be printed as JSON.
    /// Otherwise, it will be printed as plain text.
    #[arg(short, long, default_value_t = false)]
    pub as_json: bool,
    /// Output path for the history as a JSON file.
    /// Can either be a file, or a directory.
    /// If a directory is given, the filename will be
    /// STRATEGY_NAME-TIMESTAMP.json.
    #[arg(short, long)]
    pub output: Option<PathBuf>,
}

/// Generates a [`MarketRef`] instance if the given is valid, otherwise
/// it returns `None`. The market contains random quantities.
/// Valid names for markets are: `sgx`, `smse`, `tase`, and `zse`.
fn gen_market(market_name: &str) -> Option<MarketRef> {
    let market_name = market_name.to_ascii_lowercase();
    match market_name.as_str() {
        "sgx" => Some(SGX::new_random()),
        "smse" => Some(Smse::new_random()),
        "tase" => Some(TASE::new_random()),
        "zse" => Some(ZSE::new_random()),
        _ => None,
    }
}

/// Parses the given market names and returns a [`MarketRef`] if
/// available. it uses the [`gen_market`] method to
/// generate a market.
fn parse_markets(markets: &[String]) -> Vec<MarketRef> {
    let mut market_refs = Vec::new();
    let mut markets = markets
        .iter()
        .map(|m| m.to_ascii_lowercase())
        .collect::<Vec<String>>();
    // remove duplicates
    markets.dedup();
    for market_name in markets.iter() {
        if let Some(market) = gen_market(market_name.as_str()) {
            market_refs.push(market);
        } else {
            println!("Market '{market_name}' is not available. Try sgx, smse, tase, or zse.");
        }
    }
    market_refs
}

/// Tries to map the given strategy name to an optional [`StrategyIdentifier`].
/// Valid strategy names: `average-seller`.
fn map_strategy_to_id(strategy: &str) -> Option<StrategyIdentifier> {
    match strategy {
        "stingy" => Some(StrategyIdentifier::Stingy),
        "average-seller" => Some(StrategyIdentifier::AverageSeller),
        _ => None,
    }
}

/// Writes the history to the visualizer input path.
fn write_history(file_path: &PathBuf, history: &String) -> Result<(), io::Error> {
    match File::create(file_path) {
        Ok(mut file) => match file.write_all(history.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        },
        Err(e) => Err(e),
    }
}

/// Main endpoint for the executable.
fn main() {
    let args = Args::parse();

    // Init logger
    let env = Env::default().filter_or("MY_LOG_LEVEL", args.log_level);
    let _ = env_logger::try_init_from_env(env);

    let strategy_id = map_strategy_to_id(args.strategy.as_str());
    if let Some(strategy_id) = strategy_id {
        let markets = parse_markets(&args.markets);
        if markets.is_empty() {
            println!("At least one market is required");
            std::process::exit(1);
        }

        let trader = Trader::from(strategy_id, args.capital, markets);
        trader.apply_strategy(args.days, args.minute_interval);

        if let Some(mut output_path) = args.output {
            if output_path.is_dir() {
                let filename = format!("{}-{}.json", args.strategy, Local::now().timestamp());
                let filename = PathBuf::from(filename);
                output_path = output_path.join(filename);
            }

            let history = trader.get_history_as_json();
            match write_history(&output_path, &history) {
                Ok(_) => {
                    let output = output_path.as_os_str().to_str().unwrap_or_default();
                    println!("Successfully wrote history to {output}");
                }
                Err(e) => println!("Error while writing history as JSON: {}", e),
            }
        } else if args.as_json {
            println!("{}", trader.get_history_as_json());
        } else {
            println!("{:?}", trader.get_history());
        }
    } else {
        println!(
            "No strategy called '{}' available. Try: average-seller, stingy.",
            args.strategy
        );
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use crate::{gen_market, map_strategy_to_id, parse_markets};
    use trader::trader::StrategyIdentifier;

    #[test]
    fn test_parse_markets() {
        // Test with empty slice
        let markets = parse_markets(&[]);
        assert_eq!(
            0,
            markets.len(),
            "No markets should be generated for an empty slice"
        );

        // Test with no existing markets
        let names: Vec<String> = Vec::from(["a".to_string(), "b".to_string()]);
        let markets = parse_markets(&names);
        assert_eq!(
            0,
            markets.len(),
            "No markets should be generated for none existing names"
        );

        // Test with multiple existing markets
        let names: Vec<String> =
            Vec::from(["sgx".to_string(), "sgx".to_string(), "smse".to_string()]);
        let markets = parse_markets(&names);
        assert_eq!(2, markets.len(), "There shouldn't be any duplicates");

        // Test with all available markets
        let names: Vec<String> = Vec::from([
            "sgx".to_string(),
            "smse".to_string(),
            "tase".to_string(),
            "zse".to_string(),
        ]);
        let markets = parse_markets(&names);
        assert_eq!(4, markets.len(), "There must be {} markets", names.len());
    }

    #[test]
    fn test_map_strategy_to_id() {
        // test with empty str
        let id = map_strategy_to_id("");
        assert_eq!(
            None, id,
            "No identifier should be returned for an empty str"
        );

        // test non existing strategy
        let strategy = "NON-EXISTING-STRATEGY";
        let id = map_strategy_to_id(strategy);
        assert_eq!(
            None, id,
            "No identifier should be returned for the strategy {}",
            strategy
        );

        // test existing strategy
        let strategy = "average-seller";
        let expected = StrategyIdentifier::AverageSeller;
        let id = map_strategy_to_id(strategy);
        assert_eq!(
            Some(expected.clone()),
            id,
            "The id for the strategy '{}' must be '{:?}'",
            strategy,
            expected
        );
    }

    #[test]
    fn test_market_factory_gen_market() {
        // test with empty str
        let market = gen_market("");
        assert!(
            market.is_none(),
            "There should be no market for an empty name"
        );

        // test with non known name
        let market_name = "NON-EXISTING";
        let market = gen_market(market_name);
        assert!(
            market.is_none(),
            "There should be no market generated for unknown name '{}'",
            market_name
        );

        // test all known market names
        let known_names = vec!["sgx", "smse", "tase", "zse"];
        for market_name in known_names {
            let market = gen_market(market_name);
            assert!(
                market.is_some(),
                "There must be a market generated for name '{}'",
                market_name
            );
        }
    }
}
