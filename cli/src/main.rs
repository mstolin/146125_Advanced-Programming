use clap::{Parser};
use smse::Smse;
use std::cell::RefCell;
use std::rc::Rc;
use trader::trader::{StrategyIdentifier, Trader};
use unitn_market_2022::market::Market;
use SGX::market::sgx::SGX;
use TASE::TASE;
use ZSE::market::ZSE;

/// Represents a market
type MarketRef = Rc<RefCell<dyn Market>>;

#[derive(Debug, Parser)]
#[clap(about, author, version)]
pub struct Args {
    /// Name of the strategy the trader is supposed to use.
    /// Available strategy names: mostsimple.
    pub strategy: String,
    /// List of markets the trader should work with.
    /// Available market names: sgx, smse, tase, zse.
    pub markets: Vec<String>,
    /// The starting capital in EUR for the trader.
    /// The default value is 1.000.000,0 EUR.
    #[arg(short, long, default_value_t = 1_000_000.0)]
    pub capital: f32,
    /// The number of days this trader is suppose to run.
    /// By default, the trader runs for 1 day.
    #[arg(short, long, default_value_t = 1)]
    pub days: u32,
    /// Log level.
    #[arg(short, long, default_value = "error")]
    pub log_level: String,
    /// The interval of minutes the trader applies its strategy per day.
    /// By default, the trader applies its strategy every 60 minutes.
    #[arg(short, long, default_value_t = 60)]
    pub minute_interval: u32,
    /// Indicates if the history should be printed as JSON.
    /// Otherwise, it will be printed as plain text.
    #[arg(short, long, default_value_t = false)]
    pub as_json: bool,
    /// Print the history after a successful run.
    #[arg(short, long, default_value_t = true)]
    pub print_history: bool,
}

/// The `MarketFactory` is responsible to generate a `MarketRef` instance
/// for a given name.
struct MarketFactory();

impl MarketFactory {
    /// Generates a random market instance for the given name.
    /// Currently the markets sgx, smse, tase, and zse are available.
    fn gen_market(market_name: &String) -> Option<MarketRef> {
        let market_name = market_name.clone().to_ascii_lowercase();
        match market_name.as_str() {
            "sgx" => Some(SGX::new_random()),
            "smse" => Some(Smse::new_random()),
            "tase" => Some(TASE::new_random()),
            "zse" => Some(ZSE::new_random()),
            _ => None,
        }
    }
}

/// Parses the given market names and returns a `MarketRef` if
/// available. it uses the [`MarketFactory`](MarketFactory) to
/// generate a market.
fn parse_markets(markets: &[String]) -> Vec<MarketRef> {
    let mut market_refs = Vec::new();
    for market_name in markets.iter() {
        if let Some(market) = MarketFactory::gen_market(market_name) {
            market_refs.push(market);
        } else {
            // todo: print a warning that no market was found
        }
    }
    market_refs
}

/// Tries to map the given strategy name to a `StrategyIdentifier`.
fn map_strategy_to_id(strategy: String) -> Option<StrategyIdentifier> {
    match strategy.as_str() {
        "mostsimple" => Some(StrategyIdentifier::MostSimple),
        _ => None,
    }
}

fn main() {
    let args = Args::parse();

    let strategy_id = map_strategy_to_id(args.strategy);
    if let Some(strategy_id) = strategy_id {
        let markets = parse_markets(&args.markets);
        if markets.is_empty() {
            println!("At least one market is required");
            std::process::exit(1);
        }

        let trader = Trader::from(strategy_id, args.capital, markets);
        trader.apply_strategy(args.days, args.minute_interval);

        if args.print_history {
            if args.as_json {
                println!("{}", trader.get_history_as_json());
            } else {
                println!("{:?}", trader.get_history());
            }
        }
    } else {
        // TODO: Print warn that no strategy was found
        std::process::exit(1);
    }
}
