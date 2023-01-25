use crate::MarketRef;
use std::cell::RefCell;
use std::collections::HashMap;
use unitn_market_2022::good::good::Good;

pub trait Strategy {
    fn new(markets: Vec<MarketRef>) -> Self
    where
        Self: Sized;
    fn apply(&self, goods: &mut Vec<Good>, trader_name: &String);
}
