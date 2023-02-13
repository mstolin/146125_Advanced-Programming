use std::borrow::Borrow;
use log::{info, warn};
use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;
use crate::strategies::strategy::Strategy;
use crate::MarketRef;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;

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
    fn new(ex_rate: f32, good_kind: GoodKind) -> ExchangeRate {
        ExchangeRate {
            ex_rate,
            good_kind,
        }
    }
}

/// A `Deal` is a struct that save information about a specific or a possible deal with a market.
/// It saves the price and the quantity specified while searching for a deal, the kind of the good and the market
/// that owns or buys the good.
#[derive(Clone, Debug)]
struct Deal {
    /// price of the deal
    price: f32,
    /// quantity of the good
    quantity: f32,
    /// kind of the good
    good_kind: GoodKind,
    /// name of the market that sold the good
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

    /// Let the trader get the ex_rate of the Deal
    fn get_ex_rate(&self) -> f32 { self.price / self.quantity }
}


pub struct StingyStrategy {
    /// name of the trader that is using this strategy
    trader_name: String,
    /// all markets used in this strategy
    markets: Vec<MarketRef>,
    /// Price history of the exchange rate for buying goods from the markets
    /// It is a `VecDeque` in order to push back the new data and pop front the old data.
    ex_rate_buy_history: RefCell<VecDeque<ExchangeRate>>,
    /// Price history of the exchange rate for selling goods to the markets
    /// It is a `VecDeque` in order to push back the new data and pop front the old data.
    ex_rate_sell_history: RefCell<VecDeque<ExchangeRate>>,
    /// history of all the deal done by the trader for buying goods from the markets
    deals_buy_history: RefCell<Vec<Deal>>,
    /// history of all the deal done by the trader for selling goods to the markets
    deals_sell_history: RefCell<Vec<Deal>>,
}

/// buy functions
impl StingyStrategy {
    /// This function find the possible deals for buying some goods
    fn find_deals(&self, balance: f32, percentage: f32) -> Vec<Deal>{

        if percentage > 1.0 {
            warn!("percentage can't be greater than 1.0");
            return Vec::new();
        }

        let mut deals: Vec<Deal> = Vec::new();

        for market in self.markets.iter() {
            let goods = market.as_ref().borrow().get_goods();
            for good in goods {

                if good.good_kind != GoodKind::EUR {
                    let quantity = balance * percentage * good.exchange_rate_buy; /// TODO: check this
                    let buy_price = market
                        .as_ref()
                        .borrow()
                        .get_buy_price(good.good_kind, quantity);

                    if let Ok(buy_price) = buy_price {
                        // Check if the `buy_price` is greater than 0.0
                        // This because SMSE return buy prices with 0.0 as price if there's no deals
                        if buy_price > 0.0 {
                            let market_name = market.as_ref().borrow().get_name().to_string();
                            info!("Found a possible deal: {} {} at {} EUR in market: {}",
                                quantity,
                                good.good_kind,
                                buy_price,
                                market_name.clone()
                            );
                            deals.push(Deal {
                                price: buy_price,
                                quantity,
                                good_kind: good.good_kind,
                                market_name: market_name.clone()
                            });
                        }
                    } // else {
                    //     warn!("Could not find a possible deal");
                    // }
                }
            }
        }
        return deals;
    }

    /// Filter the deals contained in the `deals` vec.
    /// If there are no deal with a good ex_rate, select the deal with the ex rate that is less then the others.
    fn filter_deals(&self, deals: Vec<Deal>) -> Option<Deal> {

        let mut best_deal: Option<Deal> = None;

        let filtered_deals = deals
            .iter()
            .filter(|deal| deal.get_ex_rate() > self.get_avg_buy_ex_rate(deal.good_kind))
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

    fn buy_deal(&self, trader_goods: &mut [Good]) {
        let balance = trader_goods
            .iter_mut()
            .find(|good|good.get_kind() == GoodKind::EUR)
            .unwrap()
            .get_qty();

        let trader_eur = trader_goods
            .iter_mut()
            .find(|good|good.get_kind() == GoodKind::EUR)
            .unwrap();


        let deals= self.find_deals(balance, 0.05);
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
                    info!("Buy successful! {} {} for {} EUR from market {}",
                        buy_good.get_qty(),
                        buy_good.get_kind(),
                        deal.price,
                        deal.market_name
                    );

                    let trader_good = trader_goods
                        .iter_mut()
                        .find(|good|good.get_kind() == buy_good.get_kind())
                        .unwrap();

                    let _ = trader_good.merge(buy_good.clone());

                    // self.update_ex_rates_buy();
                    self.update_buy_history(deal);
                } else {
                    warn!("Unable to buy the good: {:?}", buy_good);
                }
            }
        }
    }
}

/// sell functions
impl StingyStrategy {

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
                                info!("Found a possible sell: {} {} at {} EUR in market: {}",
                                    quantity,
                                    good.get_kind(),
                                    sell_price,
                                    market_name.clone()
                                );
                                deals.push(Deal {
                                    price: sell_price,
                                    quantity,
                                    good_kind: good.get_kind(),
                                    market_name: market_name.clone()
                                });
                            }
                        }
                    }
                }
            }
        }
        deals
    }

    fn filter_deals_for_sell(&self, deals: Vec<Deal>) -> Option<Deal>{
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
            self.trader_name.clone()
        );

        if let Ok(token) = token {
            info!("Locked deal for sell: {} {} at {} EUR in market {}",
                deal.quantity,
                deal.good_kind,
                deal.price,
                market.get_name().to_string());
            return Some(token);
        } else {
            warn!("Could not lock the deal for sell {:?}", token);
        }

        None
    }

    fn sell_deal(&self, trader_goods: &mut [Good]) {

        let deals = self.find_deal_for_sell(trader_goods, 0.05);
        let deal = self.filter_deals_for_sell(deals);

        if let Some(deal) = deal {
            let token = self.lock_deal_for_sell(&deal);

            let market = self
                .markets
                .iter()
                .find(|market|market.as_ref().borrow().get_name().to_string() == deal.market_name)
                .unwrap();
            let mut market = market.as_ref().borrow_mut();

            if let Some(token) = token {
                let good_to_sell = trader_goods
                    .iter_mut()
                    .find(|good|good.get_kind() == deal.good_kind)
                    .unwrap();

                let sell_good = market.sell(token.clone(), good_to_sell);

                if let Ok(sell_good) = sell_good {
                    info!("Sold {} {} at {} EUR to market {}",
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

/// helper functions
impl StingyStrategy {
    /// Get the quantity of the markets "used" by the trader.
    fn get_market_qty(&self) -> usize {
        return self.markets.len();
    }

    fn display_goods(&self, trader_goods: &[Good]) {
        info!("--------- DISPLAY GOODS ---------");
        for good in trader_goods.iter() {
            info!("Trader has {} {}", good.get_qty(), good.get_kind());
        }
    }
}

/// helper functions for buying
impl StingyStrategy {
    /// Add a new exchange rate item in the [`ex_rate_history`] only if there
    /// are no more than 10 ex rate for every good kind (total: 30)
    /// If there are more than 10 ex rate for every good kind, it removes the first 3 items of
    /// the [`ex_rate_history`] deque vector.
    fn add_ex_rate_buy_to_history(&self, e: ExchangeRate) {
        let mut history = self.ex_rate_buy_history.borrow_mut();
        if history.len() >= self.get_market_qty() * 3 * 10 {
            history.pop_front();
            history.pop_front();
            history.pop_front();
        }
        history.push_back(e);
    }

    /// Return a `Vec<ExchangeRate>` that contains the exchange rate of the goods in that moment
    fn get_ex_rates_buy(&self) -> Vec<ExchangeRate> {
        let mut ex_rates: Vec<ExchangeRate> = Vec::new();
        for market in self.markets.iter() {
            let goods = market.clone().as_ref().borrow_mut().get_goods();
            for good in goods {
                if good.good_kind != GoodKind::EUR {
                    ex_rates.push(ExchangeRate::new(
                        good.exchange_rate_buy,
                        good.good_kind,
                    ));
                }
            }
        }
        return ex_rates;
    }

    /// Update the [`ex_rate_history`] with the actual exchange rate of the good using the
    /// `add_ex_rate_to_history` function
    fn update_ex_rates_buy(&self) {
        let ex_rates = self.get_ex_rates_buy();
        for item in ex_rates {
            self.add_ex_rate_buy_to_history(item);
        }
    }

    /// Return as `f32` the average exchange rate for buying a certain good kind during the last 10 operations.
    fn get_avg_buy_ex_rate(&self, good_kind: GoodKind) -> f32 {
        let mut counter = 0;
        let mut total : f32 = 0.0;

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

    fn update_buy_history(&self, deal: Deal) {
        let mut deal_buy_history = self.deals_buy_history.borrow_mut();
        deal_buy_history.push(deal);
    }
}

/// helper functions for selling
impl StingyStrategy {

    fn add_ex_rate_sell_to_history(&self, e: ExchangeRate) {
        let mut history = self.ex_rate_sell_history.borrow_mut();
        if history.len() >= self.get_market_qty() * 3 * 10 {
            history.pop_front();
            history.pop_front();
            history.pop_front();
        }
        history.push_back(e);
    }

    /// Return a `Vec<ExchangeRate>` that contains the exchange rate of the goods in that moment
    fn get_ex_rates_sell(&self) -> Vec<ExchangeRate> {
        let mut ex_rates: Vec<ExchangeRate> = Vec::new();
        for market in self.markets.iter() {
            let goods = market.as_ref().borrow().get_goods();
            for good in goods {
                if good.good_kind != GoodKind::EUR {
                    ex_rates.push(ExchangeRate::new(
                        good.exchange_rate_sell,
                        good.good_kind,
                    ));
                }
            }
        }
        return ex_rates;
    }

    fn update_ex_rates_sell(&self) {
        let ex_rates = self.get_ex_rates_sell();
        for item in ex_rates {
            self.add_ex_rate_sell_to_history(item);
        }
    }

    /// Return as `f32` the average exchange rate for buying a certain good kind during the last 10 operations.
    fn get_avg_sell_ex_rate(&self, good_kind: GoodKind) -> f32 {
        let mut counter = 0;
        let mut total : f32 = 0.0;

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

    fn update_sell_history(&self, deal: Deal) {
        let mut deal_sell_history = self.deals_sell_history.borrow_mut();
        deal_sell_history.push(deal);
    }
}

impl Strategy for StingyStrategy {
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

    fn get_markets(&self) -> &Vec<MarketRef> {
        self.markets.borrow()
    }

    fn sell_remaining_goods(&self, goods: &mut Vec<Good>) {
        // let deals = self.find_deal_for_sell(goods, 1.0);
        // for deal in deals.iter() {
        //     let token = self.lock_deal_for_sell(deal);
        //     // if let Some(token) = token {
        //     //     let market = self
        //     //         .markets
        //     //         .iter()
        //     //         .find(|market| market.as_ref().borrow().get_name() == deal.market_name)
        //     //         .unwrap();
        //     //
        //     //     let trader_good = goods
        //     //         .iter_mut()
        //     //         .find(|good|good.get_kind() == deal.good_kind)
        //     //         .unwrap();
        //     //
        //     //     let mut market = market.as_ref().borrow_mut();
        //     //     let sell_good = market.sell(token.clone(), trader_good);
        //     // }
        // }
    }

    fn apply(&self, goods: &mut Vec<Good>) {
        self.display_goods(goods);
        self.buy_deal(goods);
        self.update_ex_rates_buy();
        self.sell_deal(goods);
        self.update_ex_rates_sell();
        self.display_goods(goods);
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;
    use crate::consts::TRADER_NAME_STINGY;
    use crate::MarketRef;
    use SGX::market::sgx::SGX;
    use TASE::TASE;
    use ZSE::market::ZSE;
    use smse::Smse;
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind;
    use unitn_market_2022::market::Market;
    use crate::strategies::stingy_strategy::{Deal, StingyStrategy};
    use crate::strategies::strategy::Strategy;

    fn init_sgx(
        eur: f32,
        usd: f32,
        yen: f32,
        yuan: f32
    ) -> MarketRef {
        let sgx = SGX::new_with_quantities(eur, yen, usd, yuan);
        sgx
    }

    fn init_smse(
        eur: f32,
        usd: f32,
        yen: f32,
        yuan: f32
    ) -> MarketRef {
        let smse = Smse::new_with_quantities(eur, yen, usd, yuan);
        smse
    }

    fn init_tase(
        eur: f32,
        usd: f32,
        yen: f32,
        yuan: f32
    ) -> MarketRef {
        let tase = TASE::new_with_quantities(eur, yen, usd, yuan);
        tase
    }

    fn init_zse(
        eur: f32,
        usd: f32,
        yen: f32,
        yuan: f32
    ) -> MarketRef {
        let zse = ZSE::new_with_quantities(eur, yen, usd, yuan);
        zse
    }

    fn init_markets(
        eur: f32,
        usd: f32,
        yen: f32,
        yuan: f32
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
            Rc::clone(&zse)
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
        let markets = vec![ Rc::clone(&zse) ];
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
            assert!(token.is_some(), "There should be a token {}", token.unwrap());
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



