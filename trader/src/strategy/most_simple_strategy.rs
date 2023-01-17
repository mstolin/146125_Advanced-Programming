use crate::strategy::strategy::Strategy;
use crate::MarketRef;
use rand::seq::SliceRandom;
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::good_label::GoodLabel;

type BuyHistory = (f32, Good); // (eur buy price, bought good with bought quantuty)

pub struct MostSimpleStrategy {
    bought_goods: Vec<BuyHistory>,
}

impl MostSimpleStrategy {
    /// This method tries to find a random good with an adequate quantity that the trader can buy
    fn get_adequate_good_to_buy(&self, market: &MarketRef, eur_quantity: f32) -> Option<Good> {
        let labels = market.as_ref().borrow().get_goods().clone();
        let labels: Vec<&GoodLabel> = labels
            .iter()
            .filter(|l| l.good_kind != DEFAULT_GOOD_KIND)
            .filter(|l| l.quantity > 0.0)
            .collect();

        // get any random label
        if let Some(random_label) = labels.choose(&mut rand::thread_rng()) {
            let mut tried_quantity = random_label.quantity; // initially the max quantity
            while tried_quantity > 0.0 {
                let buy_price = market
                    .as_ref()
                    .borrow()
                    .get_buy_price(random_label.good_kind, tried_quantity)
                    .unwrap(); // todo: Care about this
                if buy_price <= eur_quantity {
                    return Some(Good::new(random_label.good_kind, tried_quantity));
                } else {
                    // divide by half
                    tried_quantity = tried_quantity / 2.0;
                }
            }
        }
        None
    }

    /// Returns the highest selling market + price for the bought good
    fn get_highest_selling_market<'a>(
        &'a self,
        markets: &'a Vec<MarketRef>,
        history: BuyHistory,
    ) -> Option<(&MarketRef, f32)> {
        markets
            .iter()
            .map(|m| {
                if let Ok(sell_price) = m
                    .as_ref()
                    .borrow()
                    .get_sell_price(history.1.get_kind(), history.1.get_qty())
                {
                    Some((m, sell_price))
                } else {
                    None
                }
            })
            .filter(|r| r.is_some())
            .map(|r| r.unwrap())
            .filter(|(_, sell_price)| *sell_price > history.0)
            .reduce(|(market_a, price_a), (market_b, price_b)| {
                if price_a > price_b {
                    (market_a, price_a)
                } else {
                    (market_b, price_b)
                }
            })
    }
}

impl Strategy for MostSimpleStrategy {
    fn new() -> Self {
        Self {
            bought_goods: Vec::new(),
        }
    }

    fn apply(&mut self, markets: &mut Vec<MarketRef>, goods: &mut Vec<Good>) {
        // ## SELL
        if !self.bought_goods.is_empty() {
            // there are still goods to sell
            /*for history in self.bought_goods {
                if let Some(market)
            }*/
        }

        // ## BUY

        // get a random market to buy from
        let market = markets.choose(&mut rand::thread_rng()).unwrap();
        // Find a random good to buy
        if let Some(good) = self.get_adequate_good_to_buy(market, 300_000.0) {
            // lock the good
            //market.as_ref().borrow_mut().lock_buy()
        }
    }
}
#[cfg(test)]
mod tests {
    use crate::strategy::most_simple_strategy::{BuyHistory, MostSimpleStrategy};
    use crate::strategy::strategy::Strategy;
    use smse::Smse;
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind;
    use unitn_market_2022::market::Market;
    use TASE::TASE;
    use ZSE::market::ZSE;

    #[test]
    fn get_highest_selling_market() {
        let quantity = 100_000.0;
        let smse = Smse::new_with_quantities(0.0, 0.0, quantity, 0.0);
        let tase = TASE::new_with_quantities(0.0, 0.0, quantity, 0.0);
        let zse = ZSE::new_with_quantities(0.0, 0.0, quantity, 0.0);

        let smse_price = smse
            .borrow()
            .get_sell_price(GoodKind::USD, quantity)
            .unwrap();
        let tase_price = tase
            .borrow()
            .get_sell_price(GoodKind::USD, quantity)
            .unwrap();
        let zse_price = zse
            .borrow()
            .get_sell_price(GoodKind::USD, quantity)
            .unwrap();

        println!(
            "SMSE: {}, TASE: {}, ZSE: {}",
            smse_price, tase_price, zse_price
        );

        let markets = Vec::from([smse, tase, zse]);

        let highest_price = Vec::from([smse_price, tase_price, zse_price])
            .iter()
            .fold(f32::INFINITY, |a, &b| a.max(b));

        // Check with
        let strategy = MostSimpleStrategy::new();
        let our_price = 0.0; // doesnt matter, but lets assume we paid nothing, so every price is higher
        let history: BuyHistory = (our_price, Good::new(GoodKind::USD, quantity));
        let (highest_selling_market, highest_market_price) = strategy
            .get_highest_selling_market(&markets, history)
            .unwrap();
        println!(
            "HIGHEST SELLING MARKET IS {} WITH {}",
            highest_selling_market.borrow().get_name(),
            highest_market_price
        );

        assert_eq!(
            highest_price, highest_market_price,
            "The highest price must be {}",
            highest_price
        );
    }

    #[test]
    fn test_get_adequate_good_to_buy() {
        let eur_quantity = 300_000.0;
        let strategy = MostSimpleStrategy::new();

        // test with no goods
        let market = ZSE::new_with_quantities(0.0, 0.0, 0.0, 0.0);
        let good = strategy.get_adequate_good_to_buy(&market, eur_quantity);
        assert_eq!(
            true,
            good.is_none(),
            "There can't be any adequate good if the market is empty"
        );

        // test with only EUR
        let market = ZSE::new_with_quantities(1_000_000.0, 0.0, 1_000_000.0, 0.0);
        assert_eq!(
            true,
            good.is_none(),
            "There can't be any adequate good if the market has only EUR"
        );

        // test with one good
        let market = ZSE::new_with_quantities(0.0, 0.0, 1_000_000.0, 0.0);
        let good = strategy.get_adequate_good_to_buy(&market, eur_quantity);
        assert_eq!(
            true,
            good.is_some(),
            "There must be at least one adequate good"
        );
        let good = good.unwrap();
        let buy_price = market
            .borrow()
            .get_buy_price(good.get_kind(), good.get_qty())
            .unwrap();
        assert_eq!(
            GoodKind::USD,
            good.get_kind(),
            "Found adequate good must be of kind USD"
        );
        assert_eq!(
            true,
            buy_price <= eur_quantity,
            "Adequate price must be equal or lower than {}",
            eur_quantity
        );

        // test with full market
        let market = ZSE::new_with_quantities(500_000.0, 500_000.0, 500_000.0, 500_000.0);
        let good = strategy.get_adequate_good_to_buy(&market, eur_quantity);
        assert_eq!(
            true,
            good.is_some(),
            "There must be at least one adequate good"
        );
        let good = good.unwrap();
        let buy_price = market
            .borrow()
            .get_buy_price(good.get_kind(), good.get_qty())
            .unwrap();
        assert!(
            good.get_kind() == GoodKind::USD
                || good.get_kind() == GoodKind::YEN
                || good.get_kind() == GoodKind::YUAN,
            "Found adequate good must be of kind USD, YEN, or YUAN"
        );
        assert_eq!(
            true,
            buy_price <= eur_quantity,
            "Adequate price must be equal or lower than {}",
            eur_quantity
        );
    }
}
