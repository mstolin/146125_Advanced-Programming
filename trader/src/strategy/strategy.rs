use crate::MarketRef;
use std::collections::HashMap;
use unitn_market_2022::good::good::Good;

pub trait Strategy {
    fn new() -> Self where Self: Sized;
    fn apply(&mut self, markets: &mut Vec<MarketRef>, goods: &mut Vec<Good>);
}
