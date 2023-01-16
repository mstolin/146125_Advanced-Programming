use unitn_market_2022::good::good::Good;
use ZSE::GoodKind;
use crate::MarketRef;
use crate::strategy::strategy::{Strategy, StrategyResult};

struct MostSimpleStrategy {
    bought_goods: Vec<GoodKind>,
}

impl Strategy {
    fn get_lowest_price_good(&self, markets: &Vec<MarketRef>) -> Good {
        markets.iter().map(|m| m.borrow().get_goods())
    }
}

impl Strategy for MostSimpleStrategy {
    fn new() -> Self {
        Self {
            bought_goods: Vec::new(),
        }
    }
    
    fn apply(&mut self, markets: &mut Vec<MarketRef>, goods: &mut Vec<Good>) {
        // ## SELLING

        // 1. Check if we own some goods to sell

        // 2. Check if we can sell it higher than we bought it

        // ## BUYING

        // 1. Try to get the good with the lowest price

        // 2. Check if we can buy it

        // 3. Buy it

    }

    fn get_result(&self) -> StrategyResult {
        todo!()
    }
}
