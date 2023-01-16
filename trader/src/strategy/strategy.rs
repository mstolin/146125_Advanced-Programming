use crate::MarketRef;
use std::collections::HashMap;
use unitn_market_2022::good::good::Good;

pub type StrategyResult = HashMap<String, i32>;

pub trait Strategy {
    fn new() -> Self;
    fn apply(&mut self, markets: &mut Vec<MarketRef>, goods: &mut Vec<Good>);
    fn get_result(&self) -> StrategyResult;
}
