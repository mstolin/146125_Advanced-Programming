use crate::MarketRef;
use std::cell::RefCell;
use std::collections::HashMap;
use unitn_market_2022::good::good::Good;

pub trait Strategy {
    /// Constructs a new trading strategy that works with the given markets.
    fn new(markets: Vec<MarketRef>) -> Self
    where
        Self: Sized;
    /// Increases the day of all given markets by one day.
    ///
    /// Call this method in trader after a single day.
    fn increase_day_by_one(&self);
    /// This methods applies the implemented strategy on the given goods.
    ///
    /// It sells goods from the given inventory.
    fn apply(&self, goods: &mut Vec<Good>, trader_name: &String);
}
