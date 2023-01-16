use crate::strategy::strategy::{Strategy, StrategyResult};
use crate::MarketRef;
use std::borrow::{Borrow, BorrowMut};
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::Market;
use unitn_market_2022::{subscribe_each_other, wait_one_day};

enum StrategyIdentifier {
    Most_Simple,
}

struct Trader {
    markets: Vec<MarketRef>,
    //strategy: Box<dyn Strategy>,
    goods: Vec<Good>,
    days: u32,
}

impl Trader {
    /// Creates a vec with all available goods
    fn create_goods(default_quantity: f32) -> Vec<Good> {
        let eur = Good::new(GoodKind::EUR, default_quantity);
        let usd = Good::new(GoodKind::USD, 0.0);
        let yen = Good::new(GoodKind::YEN, 0.0);
        let yuan = Good::new(GoodKind::YUAN, 0.0);
        Vec::from([eur, usd, yen, yuan])
    }

    /// Instantiates a trader
    pub fn new(
        strategy: StrategyIdentifier,
        start_capital: f32,
        sgx: MarketRef,
        smse: MarketRef,
        tase: MarketRef,
        zse: MarketRef,
    ) -> Self {
        if start_capital <= 0.0 {
            panic!("start_capital must be greater than 0.0")
        }

        // All markets must subscribe to each other
        subscribe_each_other!(sgx, smse, tase, zse);

        Self {
            markets: Vec::from([sgx, smse, tase, zse]),
            goods: Self::create_goods(start_capital),
            days: 0,
        }
    }
}

impl Trader {
    fn increase_day_by_one(&mut self) {
        self.days += 1;
        self.markets
            .iter_mut()
            .for_each(|m| wait_one_day!(m.as_ref()));
    }

    /**
     * Applies the strategy every *n* minutes until the day is over.
     */
    pub fn apply_strategy(&mut self, apply_every_minutes: u32) {
        let minutes_per_day: u32 = 24 * 60;
        if apply_every_minutes > minutes_per_day {
            panic!(
                "Can't apply strategy more than {} times a day (number of minutes per day)",
                minutes_per_day
            )
        }

        let mut past_minutes: u32 = 0;
        while past_minutes < minutes_per_day {
            if past_minutes % apply_every_minutes == 0 {
                // Apply strategy every n minutes
                //self.strategy.apply(&mut self.markets, &mut self.goods); // todo: Maybe internal mutability pattern here
            }
            past_minutes += 1;
        }

        self.increase_day_by_one();
    }

    pub fn get_days(&self) -> u32 {
        self.days
    }

    pub fn get_result(&self) -> StrategyResult {
        //self.strategy.get_result()
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use crate::trader::{StrategyIdentifier, Trader};
    use crate::MarketRef;
    use smse::Smse;
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind;
    use unitn_market_2022::market::Market;
    use SGX::market::sgx::SGX;
    use TASE::TASE;
    use ZSE::market::ZSE;

    fn init_markets() -> (MarketRef, MarketRef, MarketRef, MarketRef) {
        let sgx = SGX::new_random();
        let smse = Smse::new_random();
        let tase = TASE::new_random();
        let zse = ZSE::new_random();
        (sgx, smse, tase, zse)
    }

    #[test]
    fn test_new_trader() {
        let (sgx, smse, tase, zse) = init_markets();
        let trader = Trader::new(
            StrategyIdentifier::Most_Simple,
            300_000.0,
            sgx,
            smse,
            tase,
            zse,
        );
        assert_eq!(4, trader.markets.len());
        assert_eq!(4, trader.goods.len());
    }

    #[test]
    fn test_create_goods() {
        let default_qty = 300_000.0;
        let goods = Trader::create_goods(default_qty);
        assert_eq!(4, goods.len());

        let eur = Good::new(GoodKind::EUR, default_qty);
        assert_eq!(true, goods.contains(&eur), "{:?} not found in goods", eur);
        let usd = Good::new(GoodKind::USD, 0.0);
        assert_eq!(true, goods.contains(&usd), "{:?} not found in goods", usd);
        let yuan = Good::new(GoodKind::YUAN, 0.0);
        assert_eq!(true, goods.contains(&yuan), "{:?} not found in goods", yuan);
        let yen = Good::new(GoodKind::YEN, 0.0);
        assert_eq!(true, goods.contains(&yen), "{:?} not found in goods", yen);
    }
}
