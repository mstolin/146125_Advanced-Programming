use crate::consts::{TRADER_NAME_MOST_SIMPLE, TRADER_NAME_STINGY};
use crate::strategies::most_simple_strategy::MostSimpleStrategy;
use crate::strategies::strategy::Strategy;
use crate::MarketRef;
use env_logger::Env;
use serde::Serialize;
use std::cell::RefCell;

use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use crate::strategies::stingy_strategy::StingyStrategy;

#[derive(Clone, Debug, Eq, Ord, PartialOrd, PartialEq)]
pub enum StrategyIdentifier {
    MostSimple,
    Stingy,
}

#[derive(Clone, Debug, Serialize)]
pub struct HistoryDay {
    day: u32,
    eur: f32,
    usd: f32,
    yen: f32,
    yuan: f32,
}

impl HistoryDay {
    fn new(day: u32, eur: f32, usd: f32, yen: f32, yuan: f32) -> Self {
        Self {
            day,
            eur,
            usd,
            yen,
            yuan,
        }
    }
}

pub type TraderHistory = Vec<HistoryDay>;

pub struct Trader {
    /// Name of the trader
    name: String,
    /// The strategy index, the trader is using
    strategy: RefCell<Box<dyn Strategy>>,
    /// The goods the trader owns
    goods: RefCell<Vec<Good>>,
    /// History of the goods.
    /// Each row is a day (starting at day 0) and it represents the effects of the
    /// buy and sell actions, the strategy as applied.
    history: RefCell<TraderHistory>,
    /// Number of days the trader is running
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
        trader_name: &str,
    ) -> Box<dyn Strategy> {
        match id {
            StrategyIdentifier::MostSimple => {
                Box::new(MostSimpleStrategy::new(markets, trader_name))
            },
            StrategyIdentifier::Stingy => {
                Box::new(StingyStrategy::new(markets, trader_name))
            },
        }
    }

    /// Returns the name of the trader for the given strategy identifier.
    fn get_name_for_strategy(id: StrategyIdentifier) -> &'static str {
        match id {
            StrategyIdentifier::MostSimple => TRADER_NAME_MOST_SIMPLE,
            StrategyIdentifier::Stingy => TRADER_NAME_STINGY,
        }
    }

    /// Transforms a slice of goods to a [`HistoryDay`] struct.
    fn transform_good_to_history_day(day: u32, goods: &[Good]) -> HistoryDay {
        let mut day = HistoryDay::new(day, 0.0, 0.0, 0.0, 0.0);
        for good in goods.iter() {
            match good.get_kind() {
                GoodKind::EUR => day.eur = good.get_qty(),
                GoodKind::USD => day.usd = good.get_qty(),
                GoodKind::YEN => day.yen = good.get_qty(),
                GoodKind::YUAN => day.yuan = good.get_qty(),
            }
        }
        day
    }

    /// Instantiates a trader
    pub fn from(
        strategy_id: StrategyIdentifier,
        start_capital: f32,
        markets: Vec<MarketRef>,
    ) -> Self {
        if start_capital <= 0.0 {
            panic!("start_capital must be greater than 0.0")
        }
        if markets.is_empty() {
            panic!("markets can't be empty");
        }

        // Init logger
        let env = Env::default()
            .filter_or("MY_LOG_LEVEL", "info")
            .write_style_or("MY_LOG_STYLE", "always");
        let _ = env_logger::try_init_from_env(env);

        // init default goods
        let name = Self::get_name_for_strategy(StrategyIdentifier::MostSimple);
        let strategy = Self::init_strategy(strategy_id, markets, name);
        let goods = Self::create_goods(start_capital);
        let history = Vec::from([Self::transform_good_to_history_day(0, &goods)]);

        // Make all market subscribe
        strategy.subscribe_all_markets();

        Self {
            name: name.to_string(),
            strategy: RefCell::new(strategy),
            goods: RefCell::new(goods),
            history: RefCell::new(history),
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
            panic!(
                "The trader has to run at least 1 day ({} max. days given)",
                max_days
            );
        }
        if apply_every_minutes < 1 {
            panic!(
                "The trader has to be applied at least ever 1 minute instead of every {} minute/s",
                apply_every_minutes
            );
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
            let mut goods = self.goods.borrow_mut();

            // apply strategy every n minutes
            for _ in 0..interval_times {
                self.strategy.borrow_mut().apply(&mut goods);
            }

            // increase day
            *days += 1;
            self.strategy.borrow().increase_day_by_one();

            // if its the last day, sell all remaining goods
            if *days >= max_days {
                self.strategy.borrow().sell_remaining_goods(&mut goods);
            }

            // add updated goods to history after strategy has been applied
            let day = Trader::transform_good_to_history_day(*days, &goods);
            self.history.borrow_mut().push(day);
        }
    }

    /// Returns the number of days the agent is running
    pub fn get_days(&self) -> u32 {
        *self.days.borrow()
    }

    /// Returns the history of the trader
    pub fn get_history(&self) -> TraderHistory {
        self.history.borrow().clone()
    }

    /// Returns the history as a json string
    pub fn get_history_as_json(&self) -> String {
        let history = self.history.borrow();
        serde_json::to_string(&history.clone()).unwrap_or_default()
    }

    /// Returns the name of this trader
    pub fn get_name(&self) -> &String {
        &self.name
    }
}

#[cfg(test)]
mod tests {
    use crate::consts::{TRADER_NAME_MOST_SIMPLE, TRADER_NAME_STINGY};
    use crate::trader::{StrategyIdentifier, Trader};
    use crate::MarketRef;
    use smse::Smse;
    use std::collections::HashMap;
    use std::rc::Rc;
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind;
    use unitn_market_2022::market::Market;

    use SGX::market::sgx::SGX;
    use TASE::TASE;
    use ZSE::market::ZSE;

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

        // test if it works
        let trader = Trader::from(StrategyIdentifier::MostSimple, 300_000.0, markets);
        let trader_name = Trader::get_name_for_strategy(StrategyIdentifier::MostSimple);
        assert_eq!(
            trader_name,
            trader.get_name(),
            "Trader name must be equal to {}",
            trader_name
        );
        assert_eq!(
            4,
            trader.goods.borrow().len(),
            "The trader should not have more than 4 goods"
        );
        assert_eq!(0, trader.get_days(), "The trader was not running yet");
        assert_eq!(
            1,
            trader.get_history().len(),
            "The length of the history can't be bigger than 1."
        );
    }

    #[test]
    #[should_panic]
    fn test_new_trader_with_no_capital() {
        let (sgx, smse, tase, zse) = init_random_markets();
        let markets = vec![
            Rc::clone(&sgx),
            Rc::clone(&smse),
            Rc::clone(&tase),
            Rc::clone(&zse),
        ];
        Trader::from(StrategyIdentifier::MostSimple, 0.0, markets);
    }

    #[test]
    #[should_panic]
    fn test_new_trader_with_no_markets() {
        Trader::from(StrategyIdentifier::MostSimple, 300_000.0, vec![]);
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
    fn test_transform_good_to_history_day() {
        // test with empty goods
        let history = Trader::transform_good_to_history_day(12, &vec![]);
        assert_eq!(12, history.day, "Day must be {}", 12);
        assert_eq!(0.0, history.eur, "EUR must be {}", 0.0);
        assert_eq!(0.0, history.usd, "USD must be {}", 0.0);
        assert_eq!(0.0, history.yen, "YEN must be {}", 0.0);
        assert_eq!(0.0, history.yuan, "YUAN must be {}", 0.0);

        // test with a single good
        let goods = vec![Good::new(GoodKind::YEN, 30_000.0)];
        let history = Trader::transform_good_to_history_day(0, &goods);
        assert_eq!(0, history.day, "Day must be {}", 0);
        assert_eq!(0.0, history.eur, "EUR must be {}", 0.0);
        assert_eq!(0.0, history.usd, "USD must be {}", 0.0);
        assert_eq!(30_000.0, history.yen, "YEN must be {}", 30_000.0);
        assert_eq!(0.0, history.yuan, "YUAN must be {}", 0.0);

        // test with goods of the same kind
        let goods = vec![
            Good::new(GoodKind::YEN, 30_000.0),
            Good::new(GoodKind::YEN, 15_000.0),
            Good::new(GoodKind::YEN, 7_500.0),
            Good::new(GoodKind::EUR, 30_000.0),
        ];
        let history = Trader::transform_good_to_history_day(6, &goods);
        assert_eq!(6, history.day, "Day must be {}", 6);
        assert_eq!(30_000.0, history.eur, "EUR must be {}", 30_000.0);
        assert_eq!(0.0, history.usd, "USD must be {}", 0.0);
        assert_eq!(7500.0, history.yen, "YEN must be {}", 7500.0);
        assert_eq!(0.0, history.yuan, "YUAN must be {}", 0.0);

        // test with non empty goods
        let goods = vec![
            Good::new(GoodKind::EUR, 130_000.0),
            Good::new(GoodKind::USD, 10_000.0),
            Good::new(GoodKind::YEN, 30_000.0),
            Good::new(GoodKind::YUAN, 15_000.0),
        ];
        let history = Trader::transform_good_to_history_day(5, &goods);
        assert_eq!(5, history.day, "Day must be {}", 5);
        assert_eq!(130_000.0, history.eur, "EUR must be {}", 130_000.0);
        assert_eq!(10_000.0, history.usd, "USD must be {}", 10_000.0);
        assert_eq!(30_000.0, history.yen, "YEN must be {}", 30_000.0);
        assert_eq!(15_000.0, history.yuan, "YUAN must be {}", 15_000.0);
    }

    #[test]
    #[should_panic]
    fn test_apply_strategy_for_zero_days() {
        let (sgx, smse, tase, _zse) = init_random_markets();
        let markets = vec![Rc::clone(&sgx), Rc::clone(&smse), Rc::clone(&tase)];

        let trader = Trader::from(StrategyIdentifier::MostSimple, 1_000_000.0, markets);
        trader.apply_strategy(0, 0);
    }

    #[test]
    #[should_panic]
    fn test_apply_strategy_with_zero_minutes() {
        let (sgx, smse, tase, _zse) = init_random_markets();
        let markets = vec![Rc::clone(&sgx), Rc::clone(&smse), Rc::clone(&tase)];

        let trader = Trader::from(StrategyIdentifier::MostSimple, 1_000_000.0, markets);
        trader.apply_strategy(7, 0);
    }

    #[test]
    #[should_panic]
    fn test_apply_strategy_with_more_minutes_than_allowed() {
        let (sgx, smse, tase, _zse) = init_random_markets();
        let markets = vec![Rc::clone(&sgx), Rc::clone(&smse), Rc::clone(&tase)];
        let minutes = 24 * 60;

        let trader = Trader::from(StrategyIdentifier::MostSimple, 1_000_000.0, markets);
        trader.apply_strategy(7, minutes + 1);
    }

    #[test]
    fn test_get_name_for_strategy() {
        let expected = HashMap::from([(TRADER_NAME_MOST_SIMPLE, StrategyIdentifier::MostSimple), (TRADER_NAME_STINGY, StrategyIdentifier::Stingy)]);

        for (expected_name, id) in expected {
            let name = Trader::get_name_for_strategy(id.clone());
            assert_eq!(
                expected_name, name,
                "The name for id {:?} must be {}",
                id, expected_name
            );
        }
    }

    #[test]
    fn test_apply_simple_strategy_for_one_week() {
        let days = 7;
        let (sgx, smse, tase, _zse) = init_random_markets();
        let markets = vec![
            Rc::clone(&sgx),
            Rc::clone(&smse),
            Rc::clone(&tase),
            //Rc::clone(&zse), // Total "out-of-the-world" offers
        ];

        let trader = Trader::from(StrategyIdentifier::MostSimple, 1_000_000.0, markets);

        assert_eq!(0, trader.get_days(), "Trader should not have started now");
        trader.apply_strategy(7, 60);
        assert_eq!(
            days,
            trader.get_days(),
            "Trader must have been running for {} days",
            days
        );

        let history = trader.get_history();
        assert_eq!(
            days + 1,
            history.len() as u32,
            "The length of the history is supposed to be one more than the days running ({})",
            days + 1
        );
    }
}
