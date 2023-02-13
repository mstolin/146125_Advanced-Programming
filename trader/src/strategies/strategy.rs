//! This trait defines the abstraction about how a strategy is used by a trader instance.
//!
//! The goal of this implementation is to give an author of a strategy every possible freedom
//! to define what a strategy is suppose to do.
use crate::MarketRef;

use std::rc::Rc;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::{subscribe_each_other, wait_one_day};

/// Abstraction of a strategy
pub trait Strategy {
    /// Constructs a new trading strategy that works with the given markets.
    fn new(markets: Vec<MarketRef>, trader_name: &str) -> Self
    where
        Self: Sized;
    /// Returns a reference to the markets used by this strategy.
    fn get_markets(&self) -> &Vec<MarketRef>;
    /// Increases the day of all given markets by one day.
    /// Call this method after a day has passed.
    fn increase_day_by_one(&self) {
        self.get_markets()
            .iter()
            .for_each(|m| wait_one_day!(Rc::clone(m)));
    }
    /// Makes that all given markets subscribe to each other.
    /// This is required, so markets can fluctuate their prices on specific events.
    fn subscribe_all_markets(&self) {
        let markets = self.get_markets();
        for market_a in markets.iter() {
            let market_a = Rc::clone(market_a);
            let market_a_name = market_a.as_ref().borrow().get_name();
            let other_markets = markets.iter().filter(|m| {
                let market_name = m.as_ref().borrow().get_name();
                market_name != market_a_name
            });
            for market_b in other_markets {
                let market_b = Rc::clone(market_b);
                subscribe_each_other!(market_a, market_b);
            }
        }
    }
    /// When the trader stops, it is possible that other goods than EUR
    /// are still in the inventory. Maybe a strategies goal is sell everything excepts EURs.
    /// This method is supposed to be called at the end of a trader run, to sell all remaining
    /// goods **other than EUR**.
    fn sell_remaining_goods(&self, goods: &mut Vec<Good>);
    /// This methods applies the defined strategy on the given goods.
    /// The strategy is suppose to alter the given goods on sell and buy.
    fn apply(&self, goods: &mut Vec<Good>);
}
