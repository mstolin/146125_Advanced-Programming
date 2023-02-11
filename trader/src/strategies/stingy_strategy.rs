use std::cell::RefCell;
use crate::strategies::strategy::Strategy;
use crate::MarketRef;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;

/// An `ExchangeRate` is the exchange rate of a certain market in a certain moment, for a certain good
/// It will be added to a `Vec<ExchangeRate>` to keep trace of the markets exchange rate history
struct ExchangeRate {
    /// the exchange rate
    ex_rate: f32,
    /// the good kind
    good_kind: GoodKind,
    /// the market that has this exchange rate
    market_name: String,
}

impl ExchangeRate {
    fn new(ex_rate: f32, good_kind: GoodKind, market_name: String) -> ExchangeRate {
        ExchangeRate {
            ex_rate,
            good_kind,
            market_name,
        }
    }
}


pub struct StingyStrategy {
    /// name of the trader that is using this strategy
    trader_name: String,
    /// all markets used in this strategy
    markets: Vec<MarketRef>,
    /// price history of the exchange rate
    ex_rate_history: RefCell<Vec<ExchangeRate>>,
}

/// buy functions
impl StingyStrategy { }

/// helper functions
impl StingyStrategy {
    /// Get the quantity of the markets "used" by the trader.
    fn get_market_qty(&self) -> usize {
        return self.markets.len();
    }

    /// Add a new exchange rate item in the [`ex_rate_history`] only if there
    /// are no more than 10 ex rate for every good kind (total: 30)
    /// If there are more than 10 ex rate for every good kind, it removes the first 3 items of
    /// the [`ex_rate_history`] vector.
    fn add_ex_rate_to_history(&mut self, e: ExchangeRate) {
        let mut history = self.ex_rate_history.borrow_mut();
        if history.len() >= self.get_market_qty() * 3 * 10 {
            history.remove(0);
            history.remove(1);
            history.remove(2);
        }
        history.push(e);
    }

    /// Return as `f32` the average exchange rate for buying a certain good kind during the last 10 operations.
    fn get_avg_buy_ex_rate(&self, good_kind: GoodKind) -> Option<f32> {
        let mut counter = 0;
        let mut total : f32 = 0.0;

        if self.ex_rate_history.borrow().len() == 0 {
            return None;
        }
        for er in self.ex_rate_history.borrow().iter() {
            if er.good_kind == good_kind {
                total += er.ex_rate;
                counter += 1;
            }
        }

        return Some(total / counter as f32);
    }

    fn get_ex_rate(&mut self) -> Vec<ExchangeRate> {
        let mut ex_rates: Vec<ExchangeRate> = Vec::new();
        for market in self.markets.iter() {
            let goods = market.borrow().get_goods();
            for good in goods {
                if good.good_kind != GoodKind::EUR {
                    ex_rates.push(ExchangeRate::new(
                        good.exchange_rate_buy,
                        good.good_kind,
                        market.borrow().get_name().to_string()
                    ));
                }
            }
        }
        return ex_rates;
    }

    fn update_ex_rate<P>(&mut self, ex_rates: Vec<ExchangeRate> ) {
        for item in ex_rates {
            self.add_ex_rate_to_history(item);
        }
    }
}

impl Strategy for StingyStrategy {
    fn new(markets: Vec<MarketRef>, trader_name: &str) -> Self where Self: Sized {
        todo!()
    }

    fn get_markets(&self) -> &Vec<MarketRef> {
        todo!()
    }

    fn sell_remaining_goods(&self, goods: &mut Vec<Good>) {
        todo!()
    }

    fn apply(&self, goods: &mut Vec<Good>) {
        todo!()
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn something() {}
}



