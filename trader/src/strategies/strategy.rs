use crate::MarketRef;

use std::rc::Rc;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::{subscribe_each_other, wait_one_day};

pub trait Strategy {
    /// Constructs a new trading strategy that works with the given markets.
    fn new(markets: Vec<MarketRef>, trader_name: &str) -> Self
    where
        Self: Sized;
    /// Returns a reference to the markets used by this strategy.
    fn get_markets(&self) -> &Vec<MarketRef>;
    /// Increases the day of all given markets by one day.
    /// Call this method in trader after a single day.
    fn increase_day_by_one(&self) {
        self.get_markets()
            .iter()
            .for_each(|m| wait_one_day!(Rc::clone(m)));
    }
    /// Makes that all given market subscribe to each other.
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
    /// At the end, we only want EURs in our inventory.
    /// When the trader stops, it is possible that other goods than EUR
    /// are still in the inventory. This method is supposed to be called
    /// at the end of a trader run, to sell all remaining goods **other than EUR**.
    fn sell_remaining_goods(&self, goods: &mut Vec<Good>);
    /// This methods applies the implemented strategy on the given goods.
    ///
    /// It sells goods from the given inventory.
    fn apply(&self, goods: &mut Vec<Good>);
}
