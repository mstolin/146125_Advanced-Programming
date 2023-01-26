use crate::strategy::most_simple_strategy::MostSimpleStrategy;
use crate::strategy::strategy::Strategy;
use crate::MarketRef;
use std::borrow::{Borrow, BorrowMut};
use std::cell::RefCell;
use std::rc::Rc;
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::Market;
use unitn_market_2022::{subscribe_each_other, wait_one_day};
use crate::consts::TRADER_NAME_MOST_SIMPLE;

enum StrategyIdentifier {
    Most_Simple,
}

pub type TraderHistory = Vec<Vec<Good>>;

struct Trader {
    name: String,
    strategy: RefCell<Box<dyn Strategy>>,
    goods: RefCell<Vec<Good>>,
    history: RefCell<TraderHistory>,
    days: RefCell<u32>,
}

impl Trader {
    /// Creates a vec with all available goods (EUR, USD, YEN, YUAN).
    /// By default, all goods have a quantity of 0.0. Except EUR, that
    /// starts with the given default quantity that is initially defined
    /// in [`from`](from).
    fn create_goods(default_quantity: f32) -> Vec<Good> {
        let eur = Good::new(GoodKind::EUR, default_quantity);
        let usd = Good::new(GoodKind::USD, 0.0);
        let yen = Good::new(GoodKind::YEN, 0.0);
        let yuan = Good::new(GoodKind::YUAN, 0.0);
        Vec::from([eur, usd, yen, yuan])
    }

    /// Inits the strategy for the given identifier.
    fn init_strategy(
        id: StrategyIdentifier,
        markets: Vec<MarketRef>,
    ) -> RefCell<Box<dyn Strategy>> {
        match id {
            StrategyIdentifier::Most_Simple => {
                RefCell::new(Box::new(MostSimpleStrategy::new(markets)))
            }
        }
    }

    /// Returns the name of the trader for the given strategy identifier.
    fn get_name_for_strategy(id: StrategyIdentifier) -> String {
        match id {
            StrategyIdentifier::Most_Simple => TRADER_NAME_MOST_SIMPLE.to_string(),
        }
    }

    /// Instantiates a trader
    pub fn from(
        strategyId: StrategyIdentifier,
        start_capital: f32,
        markets: Vec<MarketRef>,
    ) -> Self {
        if start_capital <= 0.0 {
            panic!("start_capital must be greater than 0.0")
        }

        // init default goods
        let name = Self::get_name_for_strategy(StrategyIdentifier::Most_Simple);
        let goods = Self::create_goods(start_capital);
        let history = RefCell::new(Vec::from([goods.clone()]));

        Self {
            name,
            strategy: Self::init_strategy(strategyId, markets),
            goods: RefCell::new(goods),
            history,
            days: RefCell::new(0),
        }
    }
}

impl Trader {
    /// Applies the selected strategy every *n* minutes.
    /// It simulates minutes by calculating how many times the strategy has to be
    /// applied for a using *t = 24 * 60 / n* where *n* is defined as mentioned above.
    /// Then, it applies the strategy exactly *t* times.
    pub fn apply_strategy(&self, max_days: u32, apply_every_minutes: u32) {
        if max_days < 1 {
            panic!("The trader has to run at least 1 day ({} max. days given)", max_days);
        }

        let minutes_per_day: u32 = 24 * 60;
        if apply_every_minutes > minutes_per_day {
            panic!(
                "Can't apply strategy more than {} times a day (number of minutes per day)",
                minutes_per_day
            )
        }
        // how many times to apply the strategy per day?
        let interval_times = minutes_per_day / apply_every_minutes;
        // safe days
        let mut days = self.days.borrow_mut();

        // run the trader
        while (*days) < max_days {
            for _ in 0..interval_times {
                self.strategy
                    .borrow_mut()
                    .apply(&mut self.goods.borrow_mut(), &self.name);
                // add updated goods after strategy has been applied
                self.history.borrow_mut().push(self.goods.borrow().clone());
            }
            // lastly increase day
            *days += 1;
            self.strategy.borrow().increase_day_by_one();
        }

        // now sell all remaining goods
        self.strategy.borrow().sell_remaining_goods(&mut self.goods.borrow_mut(), &self.name);
    }

    /// Returns the number of days the agent is running
    pub fn get_days(&self) -> u32 {
        *self.days.borrow()
    }

    /// Returns the history of the trader
    pub fn get_history(&self) -> TraderHistory {
        self.history.borrow().clone()
    }
}

#[cfg(test)]
mod tests {
    use crate::trader::{StrategyIdentifier, Trader};
    use crate::MarketRef;
    use smse::Smse;
    use std::rc::Rc;
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind;
    use unitn_market_2022::market::Market;
    use unitn_market_2022::subscribe_each_other;
    use SGX::market::sgx::SGX;
    use TASE::TASE;
    use ZSE::market::ZSE;
    use crate::consts::TRADER_NAME_MOST_SIMPLE;

    fn init_random_markets() -> (MarketRef, MarketRef, MarketRef, MarketRef) {
        let sgx = SGX::new_random();
        let smse = Smse::new_random();
        let tase = TASE::new_random();
        let zse = ZSE::new_random();
        //subscribe_each_other!(&sgx, &smse, &tase, &zse); // todo fix this
        (sgx, smse, tase, zse)
    }

    #[test]
    fn test_new_trader() {
        let (sgx, smse, tase, zse) = init_random_markets();
        let markets = vec![
            Rc::clone(&sgx),
            Rc::clone(&smse),
            Rc::clone(&tase),
            Rc::clone(&zse),
        ];
        let trader = Trader::from(
            StrategyIdentifier::Most_Simple,
            300_000.0,
            markets,
        );
        let trader_name = Trader::get_name_for_strategy(StrategyIdentifier::Most_Simple);
        assert_eq!(trader_name, trader.name, "Trader name must be equal to {}", trader_name);
        assert_eq!(4, trader.goods.borrow().len(), "The trader should not have more than 4 goods");
        assert_eq!(0, trader.get_days(), "The trader was not running yet");
        assert_eq!(1, trader.history.borrow().len(), "The length of the history can't be bigger than 1.");
    }

    #[test]
    fn test_get_name_for_strategy() {
        let possible_strategies = [
            (StrategyIdentifier::Most_Simple, TRADER_NAME_MOST_SIMPLE),
        ];

        for (id, value) in possible_strategies {
            let name = Trader::get_name_for_strategy(id);
            let value = value.to_string();
            assert_eq!(value, name, "Name should be equal to constant ({})", value);
        }
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

    #[test]
    fn test_apply_strategy_for_one_week() {
        let (sgx, smse, tase, zse) = init_random_markets();
        let markets = vec![
            Rc::clone(&sgx),
            Rc::clone(&smse),
            Rc::clone(&tase),
            Rc::clone(&zse),
        ];

        let trader = Trader::from(
            StrategyIdentifier::Most_Simple,
            1_000_000.0,
            markets,
        );

        assert_eq!(0, trader.get_days(), "Trader should not have started now");
        trader.apply_strategy(1, 60);
        dbg!(trader.get_history());
        assert_eq!(7, trader.get_days(), "Trader must have been running for 7 days");

        // todo Check if all goods except EUR is 0 (Is it possible to check this?)
    }
}
