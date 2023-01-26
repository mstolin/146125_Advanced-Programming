use crate::MarketRef;
use std::cell::RefCell;
use std::rc::Rc;
use std::collections::HashMap;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::wait_one_day;

pub trait Strategy {
    /// Constructs a new trading strategy that works with the given markets.
    fn new(markets: Vec<MarketRef>) -> Self
    where
        Self: Sized;
    /// This methods applies the implemented strategy on the given goods.
    ///
    /// It sells goods from the given inventory.
    fn apply(&self, goods: &mut Vec<Good>, trader_name: &String);
    /// Returns a reference to the markets used by this strategy.
    fn get_markets(&self) -> &Vec<MarketRef>;
    /// Increases the day of all given markets by one day.
    ///
    /// Call this method in trader after a single day.
    fn increase_day_by_one(&self) {
        self.get_markets()
            .iter()
            .for_each(|m| wait_one_day!(Rc::clone(m)));
    }
}
