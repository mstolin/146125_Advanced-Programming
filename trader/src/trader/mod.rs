use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::wait_one_day;
use crate::strategy::strategy::{Strategy, StrategyResult};
use crate::MarketRef;

struct Trader {
    markets: Vec<MarketRef>,
    strategy: Box<dyn Strategy>,
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
    pub fn new(strategy: Box<dyn Strategy>, markets: Vec<MarketRef>, start_capital: f32) -> Self {
        if start_capital <= 0.0 {
            panic!("start_capital must be greater than 0.0")
        }

        Self {
            markets,
            strategy,
            goods: Self::create_goods(start_capital),
            days: 0,
        }
    }
}

impl Trader {
    fn increase_day_by_one(&mut self) {
        self.days+=1;
        self.markets.iter().for_each(|m| wait_one_day!(m));
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
                self.strategy.apply(&mut self.markets, &mut self.goods); // todo: Maybe internal mutability pattern here
            }
            past_minutes += 1;
        }

        self.increase_day_by_one();
    }

    pub fn get_days(&self) -> u32 {
        self.days
    }

    pub fn get_result(&self) -> StrategyResult {
        self.strategy.get_result()
    }
}
