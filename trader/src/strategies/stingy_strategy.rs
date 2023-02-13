use crate::strategies::strategy::Strategy;
use crate::MarketRef;
use log::{info, warn};
use std::borrow::Borrow;
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;

/// This const define the percentage that the trader is willing to buy or sell.
/// In order to be coherent with the strategy, it has not to be greater than 0.05.
pub const PERCENTAGE_BUY: f32 = 0.01;
pub const PERCENTAGE_SELL: f32 = 0.01;
pub const PERCENTAGE_SELL_ALL_GOODS: f32 = 1.0;

/// An `ExchangeRate` is struct that holds the exchange rate of a certain market in a certain moment, for a certain good
/// It will be added to a `VecDeque<ExchangeRate>` to keep trace of the markets exchange rate history
#[derive(Clone, Debug)]
struct ExchangeRate {
    /// the exchange rate
    ex_rate: f32,
    /// the good kind
    good_kind: GoodKind,
}

impl ExchangeRate {
    /// Define a new `ExchangeRate` instance
    fn new(ex_rate: f32, good_kind: GoodKind) -> ExchangeRate {
        ExchangeRate { ex_rate, good_kind }
    }
}

/// A `Deal` is a struct that save information about a possible deal with a certain market.
/// It saves the price and the quantity specified while searching for a deal, the kind of the good and the market
/// that owns or wants the good.
#[derive(Clone, Debug)]
struct Deal {
    /// price of the deal
    price: f32,
    /// quantity of the good
    quantity: f32,
    /// kind of the good
    good_kind: GoodKind,
    /// name of the market that sell or buy the good
    market_name: String,
}

impl Deal {
    /// Define a new `Deal` instance
    fn new(price: f32, quantity: f32, good_kind: GoodKind, market_name: String) -> Deal {
        Deal {
            price,
            quantity,
            good_kind,
            market_name,
        }
    }

    /// Return the exchange rate of the deal
    fn get_ex_rate(&self) -> f32 {
        self.price / self.quantity
    }
}

/// Implementation of the `StingyStrategy`.
pub struct StingyStrategy {
    /// name of the trader that use this strategy
    trader_name: String,
    /// all the markets involved in this strategy
    markets: Vec<MarketRef>,
    /// History of the exchange rates for **buying** goods from the markets.
    /// It can contain the last ten exchange rates for every kind of good.
    /// It is a `VecDeque` in order to push back the new data and pop front the old data.
    ex_rate_buy_history: RefCell<VecDeque<ExchangeRate>>,
    /// History of the exchange rates for **selling** goods to the markets.
    /// It can contain the last ten exchange rates for every kind of good.
    /// It is a `VecDeque` in order to push back the new data and pop front the old data.
    ex_rate_sell_history: RefCell<VecDeque<ExchangeRate>>,
    /// History of all the deal done by the trader for **buying** goods from the markets.
    deals_buy_history: RefCell<Vec<Deal>>,
    /// History of all the deal done by the trader for **selling** goods to the markets
    deals_sell_history: RefCell<Vec<Deal>>,
}

/// Methods for **buy**.
impl StingyStrategy {
    /// Return a `Vec<Deal>` that contains all the possible deals that the trader can do with the markets
    /// involved in the strategy.
    /// The idea is: for every market, try to find a deal spending only a little amount of EUR.
    /// This trader is **stingy**!
    fn find_deals(&self, balance: f32, percentage: f32) -> Vec<Deal> {
        if percentage > 1.0 {
            warn!("percentage can't be greater than 1.0");
            return Vec::new();
        }

        let mut deals: Vec<Deal> = Vec::new();

        for market in self.markets.iter() {
            let goods = market.as_ref().borrow().get_goods();
            for good in goods {
                if good.good_kind != GoodKind::EUR {
                    let quantity = balance * percentage * good.exchange_rate_buy;
                    /// TODO: check this
                    let buy_price = market
                        .as_ref()
                        .borrow()
                        .get_buy_price(good.good_kind, quantity);

                    if let Ok(buy_price) = buy_price {
                        // Check if the `buy_price` is greater than 0.0
                        // This because SMSE return buy prices with 0.0 as price if there's no deals
                        if buy_price > 0.0 {
                            let market_name = market.as_ref().borrow().get_name().to_string();
                            info!(
                                "Found a possible deal: {} {} at {} EUR in market: {}",
                                quantity,
                                good.good_kind,
                                buy_price,
                                market_name.clone()
                            );
                            deals.push(Deal {
                                price: buy_price,
                                quantity,
                                good_kind: good.good_kind,
                                market_name: market_name.clone(),
                            });
                        }
                    }
                }
            }
        }
        return deals;
    }

    /// Return an optional `Deal` that represent the best deal contained in the `Vec<Deal>`.
    /// A deal, to be considered good, must have an exchange rate **lower** than the average exchange rate for that
    /// type of good. If there are no deal with a "good" exchange rate, the method will select the deal that has
    /// the **lower** exchange rate in `Vec<Deal>`.
    fn filter_deals(&self, deals: Vec<Deal>) -> Option<Deal> {
        let mut best_deal: Option<Deal> = None;

        let filtered_deals = deals
            .iter()
            .filter(|deal| deal.get_ex_rate() <= self.get_avg_buy_ex_rate(deal.good_kind))
            .cloned()
            .collect::<Vec<Deal>>();

        let deals_to_iter;

        if filtered_deals.len() > 0 {
            deals_to_iter = &filtered_deals;
        } else {
            deals_to_iter = &deals;
        }
        for deal in deals_to_iter.iter() {
            if let Some(best_deal) = &mut best_deal {
                if deal.get_ex_rate() < best_deal.get_ex_rate() {
                    *best_deal = deal.clone()
                }
            } else {
                best_deal = Some(deal.clone());
            }
        }

        if best_deal.is_some() {
            info!("Found the best deal: {:?}", &best_deal);
        } else {
            warn!("Could not find the best deal");
        }
        best_deal
    }

    /// Return an optional `String` that represent the token needed to **buy** a certain quantity of a good.
    /// This method try to get a valid token for a specific deal.
    fn lock_deal(&self, deal: &Deal) -> Option<String> {
        let market = self
            .markets
            .iter()
            .find(|m| m.as_ref().borrow().get_name().to_string() == deal.market_name);

        if let Some(market) = market {
            let mut market = market.as_ref().borrow_mut();

            let token = market.lock_buy(
                deal.good_kind,
                deal.quantity,
                deal.price,
                self.trader_name.clone(),
            );

            if let Ok(token) = token {
                info!("Lock buy done with token: {}", token.clone());
                return Some(token);
            } else {
                warn!("Not able to lock buy: {:?}", token);
            }
        }

        None
    }

    /// This method try to **buy** the locked good.
    /// It uses `find_deals()` and `filter_deals()` to get a good deal, then try to lock buy using `lock_deal()`
    /// and finally buy the good from the market and merge the received amount of good.
    /// If the buy operation goes well, this method adds the deal to the buy history.
    fn buy_deal(&self, trader_goods: &mut [Good], percentage: f32) {
        let balance = trader_goods
            .iter_mut()
            .find(|good| good.get_kind() == GoodKind::EUR)
            .unwrap()
            .get_qty();

        let trader_eur = trader_goods
            .iter_mut()
            .find(|good| good.get_kind() == GoodKind::EUR)
            .unwrap();

        let deals = self.find_deals(balance, percentage);
        let deal = self.filter_deals(deals);
        if let Some(deal) = deal {
            let token = self.lock_deal(&deal);

            let market = self
                .markets
                .iter()
                .find(|market| market.as_ref().borrow().get_name().to_string() == deal.market_name)
                .unwrap();
            let mut market = market.as_ref().borrow_mut();

            if let Some(token) = token {
                let buy_good = market.buy(token.clone(), trader_eur);

                if let Ok(buy_good) = buy_good {
                    info!(
                        "Buy successful! {} {} for {} EUR from market {}",
                        buy_good.get_qty(),
                        buy_good.get_kind(),
                        deal.price,
                        deal.market_name
                    );

                    let trader_good = trader_goods
                        .iter_mut()
                        .find(|good| good.get_kind() == buy_good.get_kind())
                        .unwrap();

                    let _ = trader_good.merge(buy_good.clone());

                    self.update_buy_history(deal);
                } else {
                    warn!("Unable to buy the good: {:?}", buy_good);
                }
            }
        }
    }
}

/// Methods for **sell**.
impl StingyStrategy {
    /// Return a `Vec<Deal>` that contains all the possible deals that the trader can do with the markets
    /// involved in the strategy.
    /// The idea is: for every market, try to find a deal selling only a little amount of a certain good.
    /// This trader is **stingy**!
    fn find_deal_for_sell(&self, trader_goods: &[Good], percentage: f32) -> Vec<Deal> {
        let mut deals: Vec<Deal> = Vec::new();

        for market in self.markets.iter() {
            for good in trader_goods.iter() {
                if good.get_kind() != GoodKind::EUR {
                    let good_qty_in_market = market
                        .as_ref()
                        .borrow()
                        .get_goods()
                        .iter()
                        .find(|good_label| good_label.good_kind == good.get_kind())
                        .map(|good_label| good_label.quantity)
                        .unwrap();

                    let quantity = good.get_qty() * percentage;
                    if good_qty_in_market >= quantity {
                        let sell_price = market
                            .as_ref()
                            .borrow()
                            .get_sell_price(good.get_kind(), quantity);

                        if let Ok(sell_price) = sell_price {
                            if sell_price > 0.0 {
                                let market_name = market.as_ref().borrow().get_name().to_string();
                                info!(
                                    "Found a possible sell: {} {} at {} EUR in market: {}",
                                    quantity,
                                    good.get_kind(),
                                    sell_price,
                                    market_name.clone()
                                );
                                deals.push(Deal {
                                    price: sell_price,
                                    quantity,
                                    good_kind: good.get_kind(),
                                    market_name: market_name.clone(),
                                });
                            }
                        }
                    }
                }
            }
        }
        deals
    }

    /// Return an optional `Deal` that represent the best deal contained in the `Vec<Deal>`.
    /// A deal, to be considered good, must have an exchange rate **greater** than the average exchange rate for that
    /// type of good. If there are no deal with a "good" exchange rate, the method will select the deal that has
    /// the **higher** exchange rate in `Vec<Deal>`.
    fn filter_deals_for_sell(&self, deals: Vec<Deal>) -> Option<Deal> {
        let mut best_deal: Option<Deal> = None;

        let filtered_deals = deals
            .iter()
            .filter(|deal| deal.get_ex_rate() > self.get_avg_sell_ex_rate(deal.good_kind))
            .cloned()
            .collect::<Vec<Deal>>();

        let deals_to_iter;

        if filtered_deals.len() > 0 {
            deals_to_iter = &filtered_deals;
        } else {
            deals_to_iter = &deals;
        }

        for deal in deals_to_iter.iter() {
            if let Some(best_deal) = &mut best_deal {
                if deal.get_ex_rate() > best_deal.get_ex_rate() {
                    *best_deal = deal.clone();
                }
            } else {
                best_deal = Some(deal.clone());
            }
        }

        if best_deal.is_some() {
            info!("Found the best deal! {:?}", &best_deal);
        } else {
            warn!("Could not find the best deal");
        }

        best_deal
    }

    /// Return an optional `String` that represent the token needed to **sell** a certain quantity of a good.
    /// This method try to get a valid token for a specific deal.
    fn lock_deal_for_sell(&self, deal: &Deal) -> Option<String> {
        let market = self
            .markets
            .iter()
            .find(|m| m.as_ref().borrow().get_name() == deal.market_name)
            .map(Rc::clone)
            .unwrap();

        let mut market = market.as_ref().borrow_mut();

        let token = market.lock_sell(
            deal.good_kind,
            deal.quantity,
            deal.price,
            self.trader_name.clone(),
        );

        if let Ok(token) = token {
            info!(
                "Locked deal for sell: {} {} at {} EUR in market {}",
                deal.quantity,
                deal.good_kind,
                deal.price,
                market.get_name().to_string()
            );
            return Some(token);
        } else {
            warn!("Could not lock the deal for sell {:?}", token);
        }

        None
    }

    /// This method try to **sell** the locked good.
    /// It uses `find_deals_for_sell()` and `filter_deals_for_Sell()` to get a good deal, then try to lock sell
    /// using `lock_deal_for_sell()` and finally **sell** the good from the market and merge the received amount
    /// of good. If the sell operation goes well, this method adds the deal to the sell history.
    fn sell_deal(&self, trader_goods: &mut [Good], percentage: f32) {
        let deals = self.find_deal_for_sell(trader_goods, percentage);
        let deal = self.filter_deals_for_sell(deals);

        if let Some(deal) = deal {
            let token = self.lock_deal_for_sell(&deal);

            let market = self
                .markets
                .iter()
                .find(|market| market.as_ref().borrow().get_name().to_string() == deal.market_name)
                .unwrap();
            let mut market = market.as_ref().borrow_mut();

            if let Some(token) = token {
                let good_to_sell = trader_goods
                    .iter_mut()
                    .find(|good| good.get_kind() == deal.good_kind)
                    .unwrap();

                let sell_good = market.sell(token.clone(), good_to_sell);

                if let Ok(sell_good) = sell_good {
                    info!(
                        "Sold {} {} at {} EUR to market {}",
                        deal.quantity,
                        good_to_sell.get_kind(),
                        sell_good.get_qty(),
                        deal.market_name.clone()
                    );

                    let trader_good = trader_goods
                        .iter_mut()
                        .find(|good| good.get_kind() == sell_good.get_kind())
                        .unwrap();

                    let _ = trader_good.merge(sell_good);

                    // self.update_ex_rates_sell();
                    self.update_sell_history(deal);
                } else {
                    warn!("Unable to sell the good: {:?}", sell_good);
                }
            }
        }
    }
}

/// Helper methods
impl StingyStrategy {
    /// Get the quantity of the markets involved in this strategy.
    fn get_market_qty(&self) -> usize {
        return self.markets.len();
    }

    /// It is a method for debugging purposes.
    fn display_goods(&self, trader_goods: &[Good]) {
        info!("--------- DISPLAY GOODS ---------");
        for good in trader_goods.iter() {
            info!("Trader has {} {}", good.get_qty(), good.get_kind());
        }
    }
}

/// Helper methods for **buying**.
impl StingyStrategy {
    /// This methods dda a new exchange rate item, passed as a parameter, in the [`ex_rate_buy_history`]
    /// only if there are no more than 10 exchange rates for every kind of good (total: 30).
    /// If there are more than 10 ex rate for every kind of good, it removes the first 3 items of
    /// the [`ex_rate_buy_history`] deque vector.
    fn add_ex_rate_buy_to_history(&self, e: ExchangeRate) {
        let mut history = self.ex_rate_buy_history.borrow_mut();
        if history.len() >= self.get_market_qty() * 3 * 10 {
            history.pop_front();
            history.pop_front();
            history.pop_front();
        }
        history.push_back(e);
    }

    /// Return a `Vec<ExchangeRate>` that contains the exchange rates of the goods in this moment.
    fn get_ex_rates_buy(&self) -> Vec<ExchangeRate> {
        let mut ex_rates: Vec<ExchangeRate> = Vec::new();
        for market in self.markets.iter() {
            let goods = market.clone().as_ref().borrow_mut().get_goods();
            for good in goods {
                if good.good_kind != GoodKind::EUR {
                    ex_rates.push(ExchangeRate::new(good.exchange_rate_buy, good.good_kind));
                }
            }
        }
        return ex_rates;
    }

    /// Update the [`ex_rate_buy_history`] with the actual exchange rate of the good.
    /// It uses `add_ex_rate_buy_to_history()` and `get_ex_rates_buy()`.
    fn update_ex_rates_buy(&self) {
        let ex_rates = self.get_ex_rates_buy();
        for item in ex_rates {
            self.add_ex_rate_buy_to_history(item);
        }
    }

    /// Return as `f32` the average exchange rate for **buying** a certain good kind during the last 10 operations.
    fn get_avg_buy_ex_rate(&self, good_kind: GoodKind) -> f32 {
        let mut counter = 0;
        let mut total: f32 = 0.0;

        if self.ex_rate_buy_history.borrow().len() == 0 {
            return 0.0;
        }

        for er in self.ex_rate_buy_history.borrow().iter() {
            if er.good_kind == good_kind {
                total += er.ex_rate;
                counter += 1;
            }
        }

        return total / counter as f32;
    }

    /// This methods adds a `deal` to the buy history.
    fn update_buy_history(&self, deal: Deal) {
        let mut deal_buy_history = self.deals_buy_history.borrow_mut();
        deal_buy_history.push(deal);
    }
}

/// Helper methods for **selling**.
impl StingyStrategy {
    /// This methods dda a new exchange rate item, passed as a parameter, in the [`ex_rate_sell_history`]
    /// only if there are no more than 10 exchange rates for every kind of good (total: 30).
    /// If there are more than 10 ex rate for every kind of good, it removes the first 3 items of
    /// the [`ex_rate_sell_history`] deque vector.
    fn add_ex_rate_sell_to_history(&self, e: ExchangeRate) {
        let mut history = self.ex_rate_sell_history.borrow_mut();
        if history.len() >= self.get_market_qty() * 3 * 10 {
            history.pop_front();
            history.pop_front();
            history.pop_front();
        }
        history.push_back(e);
    }

    /// Return a `Vec<ExchangeRate>` that contains the exchange rates of the goods in this moment.
    fn get_ex_rates_sell(&self) -> Vec<ExchangeRate> {
        let mut ex_rates: Vec<ExchangeRate> = Vec::new();
        for market in self.markets.iter() {
            let goods = market.as_ref().borrow().get_goods();
            for good in goods {
                if good.good_kind != GoodKind::EUR {
                    ex_rates.push(ExchangeRate::new(good.exchange_rate_sell, good.good_kind));
                }
            }
        }
        return ex_rates;
    }

    /// Update the [`ex_rate_sell_history`] with the actual exchange rate of the good.
    /// It uses `add_ex_rate_sell_to_history()` and `get_ex_rates_sell()`.
    fn update_ex_rates_sell(&self) {
        let ex_rates = self.get_ex_rates_sell();
        for item in ex_rates {
            self.add_ex_rate_sell_to_history(item);
        }
    }

    /// Return as `f32` the average exchange rate for **selling** a certain good kind during the last 10 operations.
    fn get_avg_sell_ex_rate(&self, good_kind: GoodKind) -> f32 {
        let mut counter = 0;
        let mut total: f32 = 0.0;

        if self.ex_rate_sell_history.borrow().len() == 0 {
            return 0.0;
        }

        for er in self.ex_rate_sell_history.borrow().iter() {
            if er.good_kind == good_kind {
                total += er.ex_rate;
                counter += 1;
            }
        }

        return total / counter as f32;
    }

    /// This methods adds a `deal` to the sell history.
    fn update_sell_history(&self, deal: Deal) {
        let mut deal_sell_history = self.deals_sell_history.borrow_mut();
        deal_sell_history.push(deal);
    }
}

/// Helper methods
impl StingyStrategy {
    fn find_deal_by_good(&self, deals: &Vec<Deal>, goodkind: GoodKind) -> Vec<Deal> {
        let deals_some_good = deals
            .iter()
            .filter(|deal| deal.good_kind == goodkind)
            .cloned()
            .collect::<Vec<Deal>>();

        deals_some_good
    }
}

impl Strategy for StingyStrategy {
    /// Define a new `Strategy` instance
    fn new(markets: Vec<MarketRef>, trader_name: &str) -> Self {
        Self {
            trader_name: trader_name.to_string(),
            markets,
            ex_rate_buy_history: RefCell::new(VecDeque::new()),
            ex_rate_sell_history: RefCell::new(VecDeque::new()),
            deals_buy_history: RefCell::new(Vec::new()),
            deals_sell_history: RefCell::new(Vec::new()),
        }
    }

    /// Return a vector of `MarketRef`.
    /// This methods return references to the markets involved in the strategy.
    fn get_markets(&self) -> &Vec<MarketRef> {
        self.markets.borrow()
    }

    /// This methods try to sell all the goods owned by the trader (except for `EUR`) before closing the strategy.
    /// The assumption is: try to find an offer for every good. Since the strategy spends a little percentage of eur,
    /// it will be sufficient to try to sell for 3 times. If there are no deals for all goods, it will not to sell the remaining goods.
    fn sell_remaining_goods(&self, goods: &mut Vec<Good>) {
        for _ in 0..3 {
            self.sell_deal(goods, PERCENTAGE_SELL_ALL_GOODS);
        }
        self.display_goods(goods);
    }

    /// This method defines how to apply the strategy.
    fn apply(&self, goods: &mut Vec<Good>) {
        self.display_goods(goods);
        self.buy_deal(goods, PERCENTAGE_BUY);
        self.update_ex_rates_buy();
        self.sell_deal(goods, PERCENTAGE_SELL);
        self.update_ex_rates_sell();
        self.display_goods(goods);
    }
}

#[cfg(test)]
mod tests {
    use crate::consts::TRADER_NAME_STINGY;
    use crate::strategies::stingy_strategy::{Deal, StingyStrategy};
    use crate::strategies::strategy::Strategy;
    use crate::MarketRef;
    use smse::Smse;
    use std::rc::Rc;
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind;
    use unitn_market_2022::market::Market;
    use SGX::market::sgx::SGX;
    use TASE::TASE;
    use ZSE::market::ZSE;

    fn init_sgx(eur: f32, usd: f32, yen: f32, yuan: f32) -> MarketRef {
        let sgx = SGX::new_with_quantities(eur, yen, usd, yuan);
        sgx
    }

    fn init_smse(eur: f32, usd: f32, yen: f32, yuan: f32) -> MarketRef {
        let smse = Smse::new_with_quantities(eur, yen, usd, yuan);
        smse
    }

    fn init_tase(eur: f32, usd: f32, yen: f32, yuan: f32) -> MarketRef {
        let tase = TASE::new_with_quantities(eur, yen, usd, yuan);
        tase
    }

    fn init_zse(eur: f32, usd: f32, yen: f32, yuan: f32) -> MarketRef {
        let zse = ZSE::new_with_quantities(eur, yen, usd, yuan);
        zse
    }

    fn init_markets(
        eur: f32,
        usd: f32,
        yen: f32,
        yuan: f32,
    ) -> (MarketRef, MarketRef, MarketRef, MarketRef) {
        let sgx = SGX::new_with_quantities(eur, yen, usd, yuan);
        let smse = Smse::new_with_quantities(eur, yen, usd, yuan);
        let tase = TASE::new_with_quantities(eur, yen, usd, yuan);
        let zse = ZSE::new_with_quantities(eur, yen, usd, yuan);
        (sgx, smse, tase, zse)
    }

    #[test]
    fn test_find_deals_no_deals() {
        let trader_name = TRADER_NAME_STINGY;
        let smse = init_smse(0.0, 100.0, 100.0, 0.0);
        let tase = init_tase(0.0, 100.0, 100.0, 0.0);
        let markets = vec![Rc::clone(&smse), Rc::clone(&tase)];
        let strategy = StingyStrategy::new(markets, trader_name);

        let deals = strategy.find_deals(100_000.0, 0.05);
        assert_eq!(deals.len(), 0);
    }

    #[test]
    fn test_find_deals() {
        let trader_name = TRADER_NAME_STINGY;
        let smse = init_smse(0.0, 100_000.0, 0.0, 0.0);
        let tase = init_tase(0.0, 100_000.0, 0.0, 0.0);
        let markets = vec![Rc::clone(&smse), Rc::clone(&tase)];
        let strategy = StingyStrategy::new(markets, trader_name);

        let deal = strategy.find_deals(100_000.0, 0.05);
        assert!(deal.len() > 0, "There should be one deal");
    }

    #[test]
    fn test_filter_deals() {
        let trader_name = TRADER_NAME_STINGY;
        let (sgx, smse, tase, zse) = init_markets(0.0, 100_000.0, 0.0, 0.0);
        let markets = vec![
            Rc::clone(&sgx),
            Rc::clone(&smse),
            Rc::clone(&tase),
            Rc::clone(&zse),
        ];
        let strategy = StingyStrategy::new(markets, trader_name);

        strategy.update_ex_rates_buy();
        let deals = strategy.find_deals(100_000.0, 0.05);
        let deal = strategy.filter_deals(deals);
        assert!(deal.is_some(), "There should be a deal");
    }

    #[test]
    fn test_filter_deals_one_market() {
        let trader_name = TRADER_NAME_STINGY;
        let zse = init_zse(0.0, 100_000.0, 0.0, 0.0);
        let markets = vec![Rc::clone(&zse)];
        let strategy = StingyStrategy::new(markets, trader_name);

        strategy.update_ex_rates_buy();
        let deals = strategy.find_deals(100_000.0, 0.05);
        let deal = strategy.filter_deals(deals);
        assert!(deal.is_some(), "There should be a deal");
    }

    #[test]
    fn test_filter_deals_with_known_deals() {
        let trader_name = TRADER_NAME_STINGY;

        let usd_deal_a = Deal::new(200.0, 330.0, GoodKind::USD, "market_a".to_string());
        let usd_deal_b = Deal::new(500.0, 550.0, GoodKind::USD, "market_b".to_string());
        let usd_deal_c = Deal::new(300.0, 670.0, GoodKind::USD, "market_c".to_string());
        let usd_deal_d = Deal::new(400.0, 430.0, GoodKind::USD, "market_d".to_string());
        let yen_deal_a = Deal::new(150.0, 750.0, GoodKind::YEN, "market_a".to_string());

        let strategy = StingyStrategy::new(Vec::new(), trader_name);

        let deals = vec![usd_deal_a, usd_deal_b, usd_deal_c, usd_deal_d, yen_deal_a];
        let deal = strategy.filter_deals(deals);
        assert!(deal.is_some(), "There should be a deal");
        assert_eq!(deal.unwrap().price, 150.0);
    }

    #[test]
    fn test_lock_deal() {
        let trader_name = TRADER_NAME_STINGY;
        let sgx = init_sgx(0.0, 100_000.0, 500.0, 1000.0);
        let markets = vec![Rc::clone(&sgx)];

        let strategy = StingyStrategy::new(markets, trader_name);

        let deals = strategy.find_deals(100_000.0, 0.05);
        let deal = strategy.filter_deals(deals);
        if let Some(deal) = deal {
            let token = strategy.lock_deal(&deal);
            assert!(
                token.is_some(),
                "There should be a token {}",
                token.unwrap()
            );
        }
    }

    #[test]
    fn test_lock_deal_with_no_deal() {
        let trader_name = TRADER_NAME_STINGY;
        let sgx = init_sgx(0.0, 1.0, 1.0, 1.0);
        let markets = vec![sgx];

        let strategy = StingyStrategy::new(markets, trader_name);

        let deals = strategy.find_deals(100_000.0, 0.05);
        let deal = strategy.filter_deals(deals);
        if let Some(deal) = deal {
            let token = strategy.lock_deal(&deal);
            assert!(token.is_none(), "There should not be a token");
        }
    }

    #[test]
    fn test_find_and_filter_deals_for_sell() {
        let trader_name = TRADER_NAME_STINGY;
        let sgx = init_sgx(50.0, 50.0, 5_000.0, 1_000.0);
        let smse = init_smse(10.0, 1.0, 1.0, 0.0);
        let zse = init_zse(11.0, 1.0, 1_000.0, 1.0);

        let markets = vec![Rc::clone(&sgx), Rc::clone(&smse), Rc::clone(&zse)];

        let strategy = StingyStrategy::new(markets, trader_name);

        let good_yen = Good::new(GoodKind::YEN, 50.0);
        let good_usd = Good::new(GoodKind::USD, 90.0);
        let good_yuan = Good::new(GoodKind::YUAN, 100.0);

        let deals = strategy.find_deal_for_sell(&vec![good_yen, good_usd, good_yuan], 0.05);
        assert!(deals.len() > 0, "Should be found a good deal for sell");

        let deal = strategy.filter_deals(deals);
        assert!(deal.is_some(), "Should be found the best deal");
    }

    #[test]
    fn test_find_and_filter_deals_for_sell_with_no_deals() {
        let trader_name = TRADER_NAME_STINGY;
        let sgx = init_sgx(0.0, 0.0, 0.0, 0.0);
        let smse = init_smse(0.0, 0.0, 0.0, 0.0);
        let tase = init_tase(0.0, 0.0, 0.0, 0.0);
        // let zse = init_zse(0.0, 0.0, 0.0, 0.0);

        let markets = vec![Rc::clone(&sgx), Rc::clone(&smse), Rc::clone(&tase)];

        let strategy = StingyStrategy::new(markets, trader_name);

        let good_yen = Good::new(GoodKind::YEN, 1_000.0);
        let good_usd = Good::new(GoodKind::USD, 100.0);
        let good_yuan = Good::new(GoodKind::YUAN, 50.0);

        let deals = strategy.find_deal_for_sell(&vec![good_yen, good_usd, good_yuan], 0.05);
        assert_eq!(deals.len(), 0);

        let deal = strategy.filter_deals(deals);
        assert!(deal.is_none(), "Should not be found the best deal");
    }

    #[test]
    fn test_lock_deal_for_sell() {
        let trader_name = TRADER_NAME_STINGY;
        let sgx = init_sgx(50.0, 50.0, 5_000.0, 1_000.0);
        let smse = init_smse(10.0, 1.0, 1.0, 0.0);
        let zse = init_zse(11.0, 1.0, 1_000.0, 1.0);

        let markets = vec![Rc::clone(&sgx), Rc::clone(&smse), Rc::clone(&zse)];

        let strategy = StingyStrategy::new(markets, trader_name);

        let good_yen = Good::new(GoodKind::YEN, 50.0);
        let good_usd = Good::new(GoodKind::USD, 90.0);
        let good_yuan = Good::new(GoodKind::YUAN, 100.0);

        let deals = strategy.find_deal_for_sell(&vec![good_yen, good_usd, good_yuan], 0.05);
        let deal = strategy.filter_deals(deals);
        if let Some(deal) = deal {
            let token = strategy.lock_deal_for_sell(&deal);
            assert!(token.is_some());
        }
    }
}
