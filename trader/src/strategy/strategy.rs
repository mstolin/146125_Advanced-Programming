use crate::MarketRef;
use std::collections::HashMap;

pub type StrategyResult = HashMap<String, i32>;

pub trait Strategy {
    fn apply(&mut self, markets: &Vec<MarketRef>);
    fn get_result(&self) -> StrategyResult;
}
