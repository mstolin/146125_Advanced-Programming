use clap::Parser;
use smse::Smse;
use std::cell::RefCell;
use std::rc::Rc;
use std::thread::sleep;
use std::time::{Duration, Instant};
use unitn_market_2022::market::Market;
use SGX::market::sgx::SGX;
use TASE::TASE;
use ZSE::market::ZSE;

type MarketRef = Rc<RefCell<dyn Market>>;

struct CLI {
    strategy: String,
    max_seconds: u64,
    sleep_seconds: u64,
}

/// This will serve as the main endpoint where a strategy is being executed
fn run_trader(interval: u64, time: &Instant, markets: &Vec<MarketRef>) {
    println!(
        "This is interval {} after {} secs.",
        interval,
        time.elapsed().as_secs()
    );

    let sgx = markets
        .iter()
        .find(|m| m.borrow().get_name() == "SGX")
        .unwrap();
    let budget = sgx.borrow().get_budget();
    println!("SGX BUDGET: {}", budget);
}

fn main() {
    // parse arguments
    //let strategy =

    // Init markets
    let mut sgx = SGX::new_random();
    let mut smse = Smse::new_random();
    let mut tase = TASE::new_random();
    let mut zse = ZSE::new_random();
    let markets = Vec::from([sgx, smse, tase, zse]);
    let markets = Vec::new();

    // Definition for main loop
    let now = Instant::now();
    let run_forever = false; // true if max_seconds not set
    let max_seconds: u64 = 20; // todo: Get from user
    let sleep_time: u64 = 5; // todo: Get from user
    let max_interval = max_seconds / sleep_time;

    let mut interval = 0; // Interval counter
    while run_forever || interval < max_interval {
        run_trader(interval, &now, &markets);

        interval += 1;
        if run_forever || interval < max_interval {
            // if this is not the last round, then sleep
            sleep(Duration::new(5, 0));
        }
    }
}
