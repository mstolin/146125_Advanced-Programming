use crate::strategy::strategy::{Strategy, StrategyResult};
use crate::MarketRef;

struct Trader {
    markets: Vec<MarketRef>,
    strategy: Box<dyn Strategy>,
    days: u32,
}

impl Trader {
    /// Instantiates a trader
    pub fn new(strategy: Box<dyn Strategy>, markets: Vec<MarketRef>) -> Self {
        Self {
            markets,
            strategy,
            days: 0,
        }
    }
}

impl Trader {
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
                self.strategy.apply(&self.markets); // todo: Maybe internal mutability pattern here
            }
            past_minutes += 1;
        }

        self.days += 1;
        // todo: Increase markets days
    }

    pub fn get_days(&self) -> u32 {
        self.days
    }

    pub fn get_result(&self) -> StrategyResult {
        self.strategy.get_result()
    }
}
