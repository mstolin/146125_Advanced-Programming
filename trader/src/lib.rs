//! This the library that contains all needed functionalities to run a [`trader`] using a specific
//! [`strategies::strategy`].
//!
//! # Quick Start
//!
//! An instance of a [`trader`] using the [`strategies::average_seller_strategy`] can be created
//! like the following:
//!
//! ```rust
//! use std::rc::Rc;
//! use trader::trader::{StrategyIdentifier, Trader};
//!
//! let sgx = SGX::new_random();
//! let markets = vec![Rc::clone(&sgx)];
//!
//! let trader = Trader::from(
//!     StrategyIdentifier::AverageSeller,
//!     1_000_000.0,
//!     markets,
//! );
//!
//! trader.apply_strategy(7, 30); // Run trader for 7 days, every 30 minutes
//!
//! let history = trader.get_history(); // get the history for further computations
//! let json = trader.get_history_as_json(); // or get the history as JSON string
//! ```
//!
//! # Available Stratagies
//!
//! | Class                                   | Identifier                                    | Author        |
//! |-----------------------------------------|-----------------------------------------------|---------------|
//! | [`strategies::average_seller_strategy`] | [`trader::StrategyIdentifier::AverageSeller`] | Marcel Stolin |
//!
//! ## How to create a new strategy
//!
//! ### Step 1
//!
//! Create a new file with a meaningful name at `/src/strategy/YOUR_STRATEGY.rs`. Make your
//! new strategy public, add it to `/src/strategy/mod.rs`, and implement the
//! `Strategy` trait.
//!
//! ```rust
//! use unitn_market_2022::good::good::Good;
//! use trader::strategies::strategy::Strategy;
//!
//! pub struct YourNewStrategy {
//!     // Your custom logic here
//! }
//!
//! impl Strategy for MostSimpleStrategy {
//!     fn new(markets: Vec<MarketRef>, trader_name: &str) -> Self {
//!         // Your custom logic here
//!     }
//!
//!     fn get_markets(&self) -> &Vec<MarketRef> {
//!         // Your custom logic here
//!     }
//!
//!     fn sell_remaining_goods(&self, goods: &mut Vec<Good>) {
//!         // Your custom logic here
//!     }
//!
//!     fn apply(&self, goods: &mut Vec<Good>) {
//!         // Your custom logic here
//!     }
//! }
//! ```
//!
//! ### Step 2
//!
//! Create a new identifier for your trader by extending the enum
//! `StrategyIdentifier` at `/src/trader/mod.rs`:
//!
//! ```rust
//! enum StrategyIdentifier {
//!     Most_Simple,
//!     YOUR_NEW_TRADER, // Add your new trader
//! }
//! ```
//!
//! ### Step 3
//!
//! Define the name of your strategy at `/src/consts.rs`. The const name should
//! start with *TRADER_NAME_* to keep consistency. For example:
//!
//! ```rust
//! pub const TRADER_NAME_MOST_SIMPLE: &str = "TheMostSimpleTrader";
//! ```
//!
//! Then, add the name to `get_name_for_strategy` at `/src/trader/mod.rs`.
//! For example:
//!
//! ```rust
//! fn get_name_for_strategy(id: StrategyIdentifier) -> String {
//!     match id {
//!         StrategyIdentifier::Most_Simple => TRADER_NAME_MOST_SIMPLE.to_string(),
//!         StrategyIdentifier::YOUR_NEW_TRADER => TRADER_NAME_YOUR_NEW_TRADER.to_string(), // Add your new trader
//!     }
//! }
//! ```
//!
//! Lastly, at `/src/trader/mod.rs` extend the `test_get_name_for_strategy` test
//! case with your name:
//!
//! ```rust
//! let possible_strategies = [
//!     (StrategyIdentifier::Most_Simple, TRADER_NAME_MOST_SIMPLE),
//!     (StrategyIdentifier::YOUR_NEW_TRADER, TRADER_NAME_YOUR_NEW_TRADER), // Add your new trader
//! ];
//! // ...
//! ```
use std::cell::RefCell;
use std::rc::Rc;
use unitn_market_2022::market::Market;

mod consts;
pub mod strategies;
mod tests;
pub mod trader;

/// Representation of a market
type MarketRef = Rc<RefCell<dyn Market>>;
