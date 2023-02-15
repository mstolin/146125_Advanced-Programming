use chrono::prelude::*;
use std::fs::{File, OpenOptions};
use std::io::Write;
use unitn_market_2022::good::good_kind::GoodKind;

use crate::market::consts::{LOG_PATH, NAME};

// create a log file named log_SGX.txt
fn create_log() {
    match File::create(LOG_PATH) {
        Ok(_) => println!("File created"),
        Err(e) => println!("Unable to create the file {LOG_PATH}: {e}"),
    };
}

// open the file log_SGX.txt in read/write/append mode (so it will add a line)
fn open_log() -> File {
    let log = OpenOptions::new()
        .write(true)
        .append(true)
        .open(LOG_PATH)
        .unwrap();

    log
}

// create the log file, than open the file in read/write mode. It will delete the content and override the file.
fn init_log() -> File {
    create_log();

    let log = OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(LOG_PATH)
        .unwrap();

    log
}

// write the line to the log file, including the market name and the date like the specification requires
fn write_to_log(log_code: String) {
    let date = get_date();

    let line = format!("{}{}\n", date, log_code);

    let mut log = open_log();
    match write!(log, "{}", line) {
        Ok(_) => (),
        Err(e) => println!("Error while logging: {e}"),
    };
}

fn write_market_init(log_code: String) {
    let date = get_date();

    let line = format!("{}{}\n", date, log_code);

    let mut log = init_log();
    match write! {log, "{}", line} {
        Ok(_) => (),
        Err(e) => println!("Error while logging market init: {e}"),
    };
}

// return a string containing the date formatted like how specified in the market common plus the name of the market
// example: SGX|2022::11::20::6::30::20::1423|
pub fn get_date() -> String {
    let local: DateTime<Local> = Local::now();
    let date: String = format!(
        "{}|{}::{}::{}::{}::{}::{}::{}|",
        NAME,
        local.year(),
        local.month(),
        local.day(),
        local.hour(),
        local.minute(),
        local.second(),
        local.nanosecond()
    );

    date
}

// push to log file the initialization of the market
pub fn log_for_market_init(eur: f32, yen: f32, usd: f32, yuan: f32) {
    let log_code = format!("\nMARKET_INITIALIZATION\nEUR{:+e}\nUSD:{:+e}\nYEN:{:+e}\nYUAN:{:+e}\nEND_MARKET_INITIALIZATION",
                           eur,
                           usd,
                           yen,
                           yuan);

    write_market_init(log_code);
}

// push to log file that the lock buy function returned an error
pub fn log_for_lock_buy_err(
    trader_name: String,
    kind_to_buy: GoodKind,
    quantity_to_buy: f32,
    bid: f32,
) {
    let log_code = format!(
        "LOCK_BUY-{}-KIND_TO_BUY:{}-QUANTITY_TO_BUY:{}-BID:{}-ERROR",
        trader_name, kind_to_buy, quantity_to_buy, bid
    );

    write_to_log(log_code);
}

// push to log file that the lock buy function returned an ok
pub fn log_for_lock_buy(
    trader_name: String,
    kind_to_buy: GoodKind,
    quantity_to_buy: f32,
    bid: f32,
    token: String,
) {
    let log_code = format!(
        "LOCK_BUY-{}-KIND_TO_BUY:{}-QUANTITY_TO_BUY:{}-BID:{}-TOKEN:{}",
        trader_name, kind_to_buy, quantity_to_buy, bid, token
    );

    write_to_log(log_code);
}

// push to log file that the lock sell function returned an error
pub fn log_for_lock_sell_err(
    trader_name: String,
    kind_to_sell: GoodKind,
    quantity_to_sell: f32,
    offer: f32,
) {
    let log_code = format!(
        "LOCK-SELL-{}-KIND_TO_SELL:{}-QUANTITY_TO_SELL:{}-OFFER:{}-ERROR",
        trader_name, kind_to_sell, quantity_to_sell, offer
    );

    write_to_log(log_code);
}

// push to log file that the lock sell function returned an ok
pub fn log_for_lock_sell(
    trader_name: String,
    kind_to_sell: GoodKind,
    quantity_to_sell: f32,
    offer: f32,
    token: String,
) {
    let log_code = format!(
        "LOCK-SELL-{}-KIND_TO_SELL:{}-QUANTITY_TO_SELL:{}-OFFER:{}-TOKEN:{}",
        trader_name, kind_to_sell, quantity_to_sell, offer, token
    );

    write_to_log(log_code);
}

// push to log file that the buy function returned an error
pub fn log_for_buy_err(token: String) {
    let log_code = format!("BUY-TOKEN:{}-ERROR", token);
    write_to_log(log_code);
}

// push to log file that the buy function returned an ok
pub fn log_for_buy(token: String) {
    let log_code = format!("BUY-TOKEN:{}-OK", token);
    write_to_log(log_code);
}

// push to log file that the sell function returned an error
pub fn log_for_sell_err(token: String) {
    let log_code = format!("SELL-TOKEN:{}-ERROR", token);
    write_to_log(log_code);
}

// push to log file that the sell function returned an ok
pub fn log_for_sell(token: String) {
    let log_code = format!("SELL-TOKEN:{}-OK", token);
    write_to_log(log_code);
}
