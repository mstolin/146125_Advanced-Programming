use crate::strategy::strategy::Strategy;
use crate::MarketRef;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::borrow::{Borrow, BorrowMut};
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::HashMap;
use std::ops::Index;
use std::rc::Rc;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::good_label::GoodLabel;
use unitn_market_2022::market::Market;
use unitn_market_2022::wait_one_day;

type BuyTokenHistory = (String, f32, String); // (market name, bid, buy token)
type SellTokenHistory = (String, Good, String); // (market name, locked good, sell token)
//type BuyHistory = HashMap<GoodKind, (f32, f32)>; // GoodKind: (quantity, paid price) //(f32, GoodKind); // (eur price, bought good)
type BuyHistory = HashMap<GoodKind, Vec<f32>>;

pub struct MostSimpleStrategy {
    /// Name of the trader using this strategy
    trader_name: String,
    /// All markets this strategy works with
    markets: Vec<MarketRef>,
    /// Storage for buy tokens
    buy_tokens: RefCell<Vec<BuyTokenHistory>>,
    /// Storage for tokens that have been bought
    bought_tokens: RefCell<Vec<String>>,
    /// History of bought goods
    buy_history: RefCell<BuyHistory>,
    /// Storage for sell tokens
    sell_tokens: RefCell<Vec<SellTokenHistory>>,
    /// Storage for sold tokens
    sold_tokens: RefCell<Vec<String>>,
}

/// Buying methods
impl MostSimpleStrategy {
    /// Returns an adequate bid for the given EUR quantity.
    /// This method tries to get the maximum quantity of the good for the given label,
    /// that this strategy can buy with the given amount of EUR.
    /// It is possible that the given EUR is too low and no quantity will be found.
    ///
    /// The return value is (buy price in EUR, quantity of good).
    fn find_adequate_bid(
        &self,
        label: &GoodLabel,
        market: MarketRef,
        max_eur: f32,
    ) -> Option<(f32, f32)> {
        if label.good_kind == GoodKind::EUR {
            // Its not smart to buy eur for eur
            return None;
        }

        let market = market.as_ref().borrow();

        let mut tried_qty = label.quantity; // start with max available quantity
        let max_tries = (tried_qty / 2.0) as u32; // todo: There has to be a better solution
        let mut tries = 0;

        while tries < max_tries {
            // get cheapest price for current quantity
            if let Ok(buy_price) = market.get_buy_price(label.good_kind, tried_qty)
            {
                // is the price lower or equal to our maximum
                if buy_price <= max_eur {
                    return Some((buy_price, tried_qty));
                }
            }

            // reduce the qty if no adequate price was found
            let s = tried_qty / 2.0; // to go below zero, this has to be higher than the half
            tried_qty = tried_qty - s; // todo check for a more fine grained solution

            tries += 1;
        }
        None
    }

    /// This method tries to return the cheapest possible good from the given market.
    /// It still limits to find an adequate bid (eur price and godo quantity) for the given
    /// EUR quantity. To find an adequate bid for quantity, it uses [`find_adequate_bid()`].
    ///
    /// It may be possible that no good is buyable for the given EURs.
    ///
    /// The return value is (bid in EUR, the Good).
    fn find_cheapest_good_to_buy_from_market(
        &self,
        market: MarketRef,
        eur_qty: f32,
    ) -> Option<(f32, Good)> {
        market
            .as_ref()
            .borrow()
            .get_goods()
            .iter()
            .filter(|l| l.good_kind != GoodKind::EUR)
            .map(|label| {
                (
                    label,
                    self.find_adequate_bid(label, Rc::clone(&market), eur_qty),
                )
            })
            .filter(|(_, res)| res.is_some())
            .map(|(label, res)| {
                let (bid, qty) = res.unwrap();
                (bid, Good::new(label.good_kind, qty))
            })
            .reduce(|(bid_a, good_a), (bid_b, good_b)| {
                if good_a.get_qty() > good_b.get_qty() {
                    (bid_a, good_a)
                } else {
                    (bid_b, good_b)
                }
            })
    }

    /// This method tries to find the cheapest good from all markets for the given max. EUR
    /// quantity.
    /// To get the cheapest good for a single market, it uses the
    /// [`find_cheapest_good_from_market()`] method. For comparison, it prefers the good with the
    /// lowest bid.
    ///
    /// The return value is (market name, (bid, Good to buy)).
    fn find_cheapest_good(&self, eur_quantity: f32) -> Option<(String, (f32, Good))> {
        if eur_quantity <= 0.0 {
            return None;
        }
        self.markets
            .iter()
            .map(|m| {
                let market = Rc::clone(m);
                let market = (*market).borrow();
                (
                    market.get_name().to_string(),
                    self.find_cheapest_good_to_buy_from_market(Rc::clone(m), eur_quantity),
                )
            })
            .filter(|(m, res)| res.is_some())
            .map(|(m, res)| (m, (res.unwrap())))
            .reduce(|(m_a, (bid_a, good_a)), (m_b, (bid_b, good_b))| {
                // returns the cheapest good
                if bid_a < bid_b {
                    (m_a, (bid_a, good_a))
                } else {
                    (m_b, (bid_b, good_b))
                }
            })
    }

    fn lock_cheapest_good_for_buy(&self, inventory: &Vec<Good>) {
        let mut buy_tokens = self.buy_tokens.borrow_mut();
        // 1. Find cheapest good to buy
        let mut eur_qty = self
            .get_good_for_kind(GoodKind::EUR, inventory)
            .unwrap()
            .get_qty();
        let cheapest_good = self.find_cheapest_good(eur_qty);
        // 2. If a cheapest good has been found, try to lock it
        if let Some((market_name, (bid, good))) = cheapest_good {
            // We can be sure the market exist
            let market = self.find_market_for_name(&market_name).unwrap();
            let mut market = market.as_ref().borrow_mut();

            if market.get_name() == market_name {
                // 2. Lock good to buy
                let token = market.lock_buy(
                    good.get_kind(),
                    good.get_qty(),
                    bid,
                    self.trader_name.clone(),
                );
                if let Ok(token) = token {
                    buy_tokens.push((market_name.clone(), bid, token.clone()));
                }
            }
        }
    }

    fn buy_locked_goods(&self, inventory: &mut Vec<Good>) {
        let mut buy_tokens = self.buy_tokens.borrow_mut();
        let mut bought_tokens = self.bought_tokens.borrow_mut();

        // loop over all buy tokens and buy the goods
        for (market_name, bid, token) in buy_tokens.iter() {
            // borrow market as mut
            let market = self.find_market_for_name(market_name).unwrap();
            let mut market = market.as_ref().borrow_mut();

            let mut eur = self
                .get_mut_good_for_kind(GoodKind::EUR, inventory)
                .unwrap();
            if let Ok(bought_good) = market.buy(token.clone(), eur) {
                println!("SUCCESSFULLY BOUGHT {} {} FOR {} EUR AT {}", bought_good.get_qty(), bought_good.get_kind(), bid, market.get_name());
                self.add_to_buy_history(&bought_good, *bid);
                // todo Better error handling
                let mut our_good = self
                    .get_mut_good_for_kind(bought_good.get_kind(), inventory)
                    .unwrap();
                let _ = our_good.merge(bought_good.clone());
                bought_tokens.push(token.clone());
            }
        }
    }

    /// This method adds the average price of the current buy to the history.
    /// Call this method after a successful buy.
    fn add_to_buy_history(&self, bought_good: &Good, bid: f32) {
        let mut buy_history = self.buy_history.borrow_mut();
        let mut avg_vec = buy_history.get_mut(&bought_good.get_kind()).unwrap();
        let avg = bid / bought_good.get_qty();
        avg_vec.push(avg);
    }

    /// This method removes the quantity and the offer from the kind history.
    /// It should be called after a good has been successful sold.
    /*fn remove_from_buy_history(&self, sold_good: &Good, offer: f32) {
        let mut buy_history = self.buy_history.borrow_mut();
        let (hist_qty, hist_paid) = buy_history.get_mut(&sold_good.get_kind()).unwrap();
        *hist_qty-=sold_good.get_qty();
        *hist_paid-=offer;
    }*/

    fn clear_bought_tokens(&self) {
        let mut buy_tokens = self.buy_tokens.borrow_mut();
        let mut bought_tokens = self.bought_tokens.borrow();

        for bought_token in bought_tokens.iter() {
            if let Some(index) = buy_tokens
                .iter()
                .position(|(_, _, token)| *token == *bought_token)
            {
                // token was found -> remove it
                buy_tokens.remove(index);
            }
        }

        // todo: Clear bought tokens ??
    }
}

/// Selling Methods
impl MostSimpleStrategy {
    /// This method tries to find the highest selling market for the given good.
    ///
    /// It first checks if the offer of the market is at least higher than the buy
    /// price. Then, it returns the market with the highest offer.
    /*fn find_highest_selling_market_for_good(
        &self,
        good: &Good,
    ) -> Option<(String, f32)> {
        self.markets
            .iter()
            .map(|m| {
                (
                    m.as_ref().borrow().get_name().to_string(),
                    self.find_adequate_offer(Rc::clone(m), good)
                )
            })
            .filter(|(_, offer)| offer.is_some())
            .map(|(name, offer)| (name, offer.unwrap()))
            .reduce(|(market_a, offer_a), (market_b, offer_b)| {
                if offer_a > offer_b {
                    // todo: WHY IS THE MARKET MORE EFFECTIVE IF offer_a < offer_b ???
                    (market_a, offer_a)
                } else {
                    (market_b, offer_b)
                }
            })
    }*/

    /// Returns (offer, quantity)
    fn find_adequate_offer(&self, market: MarketRef, good: &Good) -> Option<(f32, f32)> {
        if good.get_kind() == GoodKind::EUR || good.get_qty() <= 0.0 {
            return None;
        }

        let market =  market.as_ref().borrow();
        let average_buy_price = self.get_average_price_for_good(&good.get_kind());

        //println!("TRY TO FIND ADEQUATE OFFER FOR {} {} AT {} EUR AVG AT MARKET {}", good.get_qty(), good.get_kind(), average_buy_price, market.get_name());

        // By default, start with max quantity available
        let mut quantity = good.get_qty();
        let max_tries = 20;//(quantity / 2.0) as u32; // todo: There has to be a better solution
        let mut tries: u32 = 0;

        while tries < max_tries {
            let sell_price = market.get_sell_price(good.get_kind(), quantity);
            if let Ok(sell_price) = sell_price {
                let avg = sell_price / quantity;
                //println!("AVG IS {} EUR FOR {} {} (OUR AVG {} EUR)", last_avg, quantity, good.get_kind(), average_buy_price);
                // try find an avg. sell price that is higher than our avg. buy price to make profit
                if avg > average_buy_price {
                    println!("FOUND AN ADEQUATE OFFER: {} EUR FOR {} {} AT LAST AVG {} EUR AT MARKET {}", sell_price, quantity, good.get_kind(), avg, market.get_name());
                    return Some((sell_price, quantity));
                }
            } else {
                println!("!!!!!!!! WE HAVE AN ERROR");
                dbg!(sell_price);
            }

            // no good price for current quantity has been found, so lower the quantity to try with
            let s = quantity / 2.0;
            quantity = quantity - s; // todo check for a more fine grained solution
            tries += 1;
        }

        None
    }

    fn lock_goods_for_sell(&self, inventory: &mut Vec<Good>) {
        let mut offers = HashMap::new();

        // 1. Find the quantity we can sell with the highest profit for that market for every good
        for market in self.markets.iter() {
            let mut market_offers = HashMap::new();
            for good in inventory.iter() {
                if good.get_kind() == GoodKind::EUR {
                    // we don't want to sell EUR
                    continue;
                }

                let offer = self.find_adequate_offer(Rc::clone(market), good);
                if let Some(offer) = offer {
                    println!("FOUND OFFER OF {} EUR FOR {} {} AT {}", offer.0, offer.1, good.get_kind(), market.as_ref().borrow().get_name());
                    market_offers.insert(good.get_kind(), offer);
                }
            }

            let market_name = market.as_ref().borrow().get_name().to_string();
            offers.insert(market_name.clone(), market_offers);
        }

        // 2. Find the best offer for every good
        let mut best_offers = HashMap::new();
        for (market_name, market_offers) in offers.iter() {
            // try to find the best offer for the current good
            for (kind, (price, quantity)) in market_offers.iter() {
                let mut best_offer = best_offers.get_mut(kind);
                // Does a best offer already exist?
                if let Some((best_market, best_price, best_quantity)) = best_offer {
                    // Some best offer already exists, so compare it
                    if price > best_price {
                        // found a new best price => update
                        *best_market = market_name.clone();
                        *best_price = *price;
                        *best_quantity = *quantity;
                    }
                } else {
                    // has no offer yet, so insert current offer
                    best_offers.insert(kind.clone(), (market_name.clone(), *price, *quantity));
                }
            }

        }

        // 3. Lock best offers
        for (kind, (market_name, offer, quantity)) in best_offers.iter() {
            // We can be sure, this market exist
            let market = self.markets.iter().find(|m| m.as_ref().borrow().get_name().to_string() == *market_name).map(|m| Rc::clone(m)).unwrap();
            let mut market = market.as_ref().borrow_mut();

            // try to lock it
            let token = market.lock_sell(kind.clone(), *quantity, *offer, self.trader_name.clone());
            if let Ok(token) = token {
                // lock was successful, save token
                let good_to_lock = Good::new(kind.clone(), *quantity);
                println!("LOCKED {:?} FOR {} EUR", good_to_lock, offer);
                self.sell_tokens
                    .borrow_mut()
                    .push((market_name.clone(), good_to_lock, token)); // TODO: Make custom struct Offer { }
            } else {
                // For whatever reason, market does not like to lock the good
                // Therefore, we have to try the second best offer if available
                println!("!!! MARKET {}: {:?}", market_name, token); // TODO ERROR InsufficientDefaultGoodQuantityAvailable

                // 1. Remove this offer from best offers

                // 2. Use recursion to retry
            }
        }

        // 4. Remove all offers we can't sell anymore and repeat step 3


        // try to sell every good we own
        /*for good in inventory {
            // ... except EUR
            if good.get_kind() == GoodKind::EUR {
                continue;
            }

            // Try find the highest selling market for an adequate quantity
            let highest_selling_market = self.find_highest_selling_market_for_good(good);
            if let Some((market_name, sell_price)) = highest_selling_market {
                // now need to lock the good (can be sure that this market exist)
                let market = self.find_market_for_name(&market_name).unwrap();
                let mut market = market.as_ref().borrow_mut();

                //let token = market.lock_sell()
            }
        }*/

        /*for (good_kind, _) in self.buy_history.borrow().iter() {
            // Don't sell EUR
            if *good_kind == GoodKind::EUR {
                continue;
            }

            // we can be sure that this good exist
            let good = self.get_mut_good_for_kind(good_kind.clone(), inventory).unwrap();
            // Find the market with highest sell price
            if let Some((market_name, sell_price)) =
                self.find_highest_selling_market_for_good(good)
            {
                // found an adequate market, now need to lock the good (can be sure that this market exist)
                let market = self.find_market_for_name(&market_name).unwrap();
                let mut market = market.as_ref().borrow_mut();

                // now lock the good
                if let Ok(token) = market.lock_sell(
                    good.get_kind(),
                    good.get_qty(),
                    sell_price,
                    self.trader_name.clone(),
                ) {
                    // successfully locked good for sell, add token with price to the sell token history
                    self.sell_tokens
                        .borrow_mut()
                        .push((market_name, good.clone(), token));
                }
            }
        }*/
    }

    fn sell_locked_goods(&self, inventory: &mut Vec<Good>) {
        let mut sold_tokens = self.sold_tokens.borrow_mut();
        let mut sell_tokens = self.sell_tokens.borrow_mut();

        // loop over all sell tokens and sell the good
        for (market_name, good, token) in sell_tokens.iter_mut() {
            // We can be sure that this market exist
            let market = self.find_market_for_name(&market_name).unwrap();
            let mut market = market.as_ref().borrow_mut();

            if let Ok(cash) = market.sell(token.clone(), good) {
                // Now increase our eur quantity
                let mut eur = self
                    .get_mut_good_for_kind(GoodKind::EUR, inventory)
                    .unwrap();
                let _ = eur.merge(cash); // todo handle the error
                sold_tokens.push(token.clone());
            }
        }
    }

    // todo this redundant
    fn clear_sold_tokens(&self) {
        let mut sell_tokens = self.sell_tokens.borrow_mut();
        let mut sold_tokens = self.sold_tokens.borrow();

        for sold_token in sold_tokens.iter() {
            if let Some(index) = sell_tokens
                .iter()
                .position(|(_, _, token)| *token == *sold_token)
            {
                // token exist
                sell_tokens.remove(index);
            }
        }
    }
}

/// Helper methods
impl MostSimpleStrategy {
    fn init_default_buy_history() -> BuyHistory {
        // don't care about EUR
        let kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];

        let mut history: BuyHistory = HashMap::new();
        for kind in kinds {
            history.insert(kind, Vec::new());
        }

        history
    }

    fn increase_eur_qty(&self, goods: &mut Vec<Good>, merge_eur: Good) {
        let eur = goods.iter_mut().find(|g| g.get_kind() == GoodKind::EUR);
        if let Some(eur) = eur {
            let _ = eur.merge(merge_eur);
        }
    }

    fn get_mut_good_for_kind<'a>(
        &'a self,
        kind: GoodKind,
        inventory: &'a mut Vec<Good>,
    ) -> Option<&mut Good> {
        inventory.iter_mut().find(|g| g.get_kind() == kind)
    }

    fn get_good_for_kind<'a>(&'a self, kind: GoodKind, inventory: &'a Vec<Good>) -> Option<&Good> {
        inventory.iter().find(|g| g.get_kind() == kind)
    }

    fn find_market_for_name(&self, name: &String) -> Option<&MarketRef> {
        self.markets
            .iter()
            .find(|m| m.as_ref().borrow().get_name().to_string() == *name)
    }

    fn get_average_price_for_good(&self, kind: &GoodKind) -> f32 {
        let buy_history = self.buy_history.borrow();
        let avg_vec = buy_history.get(kind).unwrap();
        let sum: f32 = avg_vec.iter().sum();
        sum / (avg_vec.len() as f32)
    }
}

/// Strategy trait implementation
impl Strategy for MostSimpleStrategy {
    fn new(markets: Vec<MarketRef>, trader_name: &String) -> Self {
        Self {
            trader_name: trader_name.clone(),
            markets,
            buy_tokens: RefCell::new(Vec::new()),
            bought_tokens: RefCell::new(Vec::new()),
            buy_history: RefCell::new(MostSimpleStrategy::init_default_buy_history()),
            sell_tokens: RefCell::new(Vec::new()),
            sold_tokens: RefCell::new(Vec::new()),
        }
    }

    fn get_markets(&self) -> &Vec<MarketRef> {
        self.markets.borrow()
    }

    fn sell_remaining_goods(&self, goods: &mut Vec<Good>) {
        ()
        /*let mut cash_qty: f32 = 0.0;
        for good in goods.iter_mut() {
            if good.get_kind() == GoodKind::EUR || good.get_qty() == 0.0 {
                continue;
            }
            if let Some((market_name, offer)) = self.find_highest_selling_market_for_good(good, 0.0)
            {
                let market = self.find_market_for_name(&market_name).unwrap();
                let mut market = market.as_ref().borrow_mut();
                if let Ok(token) = market.lock_sell(
                    good.get_kind(),
                    good.get_qty(),
                    offer,
                    self.trader_name.clone(),
                ) {
                    if let Ok(cash) = market.sell(token, good) {
                        // todo this is redundant as in sell method like above
                        cash_qty += cash.get_qty();
                    }
                }
            }
        }*/

        /*
        Her we do something like a bank.
        Above, if the good was sold, add the cash to an array.
        Down here, sum all of the money and add it to our cash.
        todo: Make this reusable in all other function e.g. increase_eur()
         */
        /*let mut eur = self.get_mut_good_for_kind(GoodKind::EUR, goods).unwrap();
        let cash = Good::new(GoodKind::EUR, cash_qty);
        let _ = eur.merge(cash); // todo handle the error*/
    }

    fn apply(&self, goods: &mut Vec<Good>) {
        self.lock_cheapest_good_for_buy(goods); // 1. Lock buy the cheapest good we can find
        self.buy_locked_goods(goods); // 2. Buy all locked goods
        self.clear_bought_tokens(); // 3. Clear buy tokens
        self.lock_goods_for_sell(goods); // 4. Lock sell all goods for a higher price
        self.sell_locked_goods(goods); // 5. Sell our goods
        self.clear_sold_tokens(); // 6. Clear sell tokens
    }
}

/*#[cfg(test)]
mod tests {
    use crate::strategy::most_simple_strategy::MostSimpleStrategy;
    use crate::strategy::strategy::Strategy;
    use crate::MarketRef;
    use smse::Smse;
    use std::borrow::Borrow;
    use std::rc::Rc;
    use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind;
    use unitn_market_2022::market::Market;
    use SGX::market::sgx::SGX;
    use TASE::TASE;
    use ZSE::market::ZSE;

    #[test]
    fn test_find_highest_selling_market_for_good() {
        let quantity = 100_000.0;
        let smse = Smse::new_with_quantities(0.0, 0.0, quantity, 0.0);
        let tase = TASE::new_with_quantities(0.0, 0.0, quantity, 0.0);
        let zse = ZSE::new_with_quantities(0.0, 0.0, quantity, 0.0);

        let bought_qty = 10_000.0;
        let zse_sell_price = zse
            .as_ref()
            .borrow()
            .get_sell_price(GoodKind::USD, bought_qty)
            .unwrap();

        let zse_name = zse.as_ref().borrow().get_name();

        let markets = vec![Rc::clone(&zse)];
        let trader_name = "TEST_TRADER".to_string();
        let strategy = MostSimpleStrategy::new(markets, &trader_name);

        // test with only one market
        let bought_good = Good::new(GoodKind::USD, bought_qty);
        let buy_price = zse_sell_price * 0.8; // a little bit less than the sell price
        let res = strategy.find_highest_selling_market_for_good(&bought_good, buy_price);
        assert_eq!(
            true,
            res.is_some(),
            "At least one highest selling market should be found"
        );
        let (market_name, highest_sell_price) = res.unwrap();
        assert_eq!(
            zse_name, market_name,
            "Highest selling market should be {}",
            zse_name
        );
        assert_eq!(
            zse_sell_price, highest_sell_price,
            "Highest found sell price {} should be equal to {}",
            highest_sell_price, zse_sell_price
        );

        // test with one market but way too high buy price
        let buy_price = zse_sell_price * 2.0;
        let res = strategy.find_highest_selling_market_for_good(&bought_good, buy_price);
        assert_ne!(
            true,
            res.is_some(),
            "No selling offer should be found for buy price {}",
            buy_price
        );

        // test with multiple markets
        let markets = vec![Rc::clone(&smse), Rc::clone(&tase), Rc::clone(&zse)];
        let trader_name = "TEST_TRADER".to_string();
        let strategy = MostSimpleStrategy::new(markets, &trader_name);
        let buy_price = zse_sell_price * 0.8; // then at least zse should be found
        let res = strategy.find_highest_selling_market_for_good(&bought_good, buy_price);
        assert_eq!(
            true,
            res.is_some(),
            "With multiple markets, there should be at least one selling market"
        );
        let (market_name, highest_sell_price) = res.unwrap();
        let markets = vec![Rc::clone(&smse), Rc::clone(&tase), Rc::clone(&zse)];
        let highest_selling_market = markets
            .iter()
            .find(|m| m.as_ref().borrow().get_name() == market_name)
            .unwrap();
        let highest_selling_offer = highest_selling_market
            .borrow_mut()
            .get_sell_price(bought_good.get_kind(), bought_good.get_qty())
            .unwrap();
        assert_eq!(
            highest_selling_offer, highest_sell_price,
            "The highest selling price should be {}",
            highest_selling_offer
        );
    }

    /*#[test]
    fn test_sell_if_needed() {
        let market = ZSE::new_with_quantities(500_000.0, 500_000.0, 500_000.0, 500_000.0);
        let usd_sell_price = market
            .borrow()
            .get_sell_price(GoodKind::USD, 20_000.0)
            .unwrap();
        let mut markets = Vec::from([market]);

        let mut our_goods = Vec::from([
            Good::new(GoodKind::EUR, 0.0),
            Good::new(GoodKind::USD, 0.0),
            Good::new(GoodKind::YEN, 0.0),
            Good::new(GoodKind::YUAN, 0.0),
        ]);
        let mut strategy = MostSimpleStrategy {
            buy_history: Vec::from([(usd_sell_price - 1.0, Good::new(GoodKind::USD, 20_000.0))]),
            buy_tokens: Vec::new(),
        };
        strategy.sell_if_needed(&mut markets, &mut our_goods, &"TEST_TRADER".to_string());

        let new_eur = our_goods
            .iter()
            .find(|g| g.get_kind() == DEFAULT_GOOD_KIND)
            .unwrap();
        assert_eq!(
            usd_sell_price,
            new_eur.get_qty(),
            "After selling, EUR has to be {}",
            usd_sell_price
        );
    }*/

    #[test]
    fn test_find_cheapest_good_to_buy_from_market() {
        let quantity: f32 = 1_000.0;
        let zse = ZSE::new_with_quantities(quantity, quantity, quantity, quantity);

        let trader_name = "TEST_TRADER".to_string();
        let strategy = MostSimpleStrategy::new(vec![Rc::clone(&zse)], &trader_name);
        let zse = Rc::clone(&zse);

        //let found_good = strategy.find_cheapest_good_to_buy_from_market(Rc::clone(&market), quantity);
        let found_good = strategy.find_cheapest_good_to_buy_from_market(Rc::clone(&zse), quantity);
        assert_eq!(
            false,
            found_good.is_none(),
            "There has to be one cheapest good"
        );

        let (found_price, found_good) = found_good.unwrap();
        assert_ne!(
            found_good.get_kind(),
            DEFAULT_GOOD_KIND,
            "Found Good can't be of kind {}",
            DEFAULT_GOOD_KIND
        );
        assert!(
            found_price <= quantity,
            "The found price can't be higher than owned amount of EUR {}",
            quantity
        );

        let market_good = zse
            .as_ref()
            .borrow()
            .get_goods()
            .iter()
            .find(|g| g.good_kind == found_good.get_kind())
            .unwrap()
            .clone();
        assert!(
            market_good.quantity >= found_good.get_qty(),
            "Found quantity can't be higher than the available quantity of {} {}",
            market_good.quantity,
            market_good.good_kind
        );

        let market_price = zse
            .as_ref()
            .borrow()
            .get_buy_price(found_good.get_kind(), found_good.get_qty())
            .unwrap();
        assert_eq!(
            market_price, found_price,
            "Price of found good should be {}",
            market_price
        );
    }

    /*#[test]
    fn test_find_adequate_bid() {
        let quantity: f32 = 1_000.0;
        let market = ZSE::new_with_quantities(quantity, quantity, quantity, quantity);
        let strategy = MostSimpleStrategy::new(vec![Rc::clone(&market)]);

        let goods = market.as_ref().borrow().get_goods();

        let mut iter = goods.iter().filter(|l| l.good_kind != DEFAULT_GOOD_KIND);
        let high_eur = 1_000_000.0; // test with very high bid
        while let Some(label) = iter.next() {
            let (price, qty) = strategy
                .find_adequate_bid(label, Rc::clone(&market), high_eur)
                .unwrap();
            assert!(
                price <= high_eur,
                "Adequate buy price can't be higher than {}",
                high_eur
            );
            assert!(
                qty > 0.0,
                "Quantity ({}) of adequate bid can't be less or equal to 0",
                qty
            );
        }
    }*/

    #[test]
    fn test_find_cheapest_good() {
        let quantity = 100_000.0;
        let smse = Smse::new_with_quantities(quantity, quantity, quantity, quantity);
        let tase = TASE::new_with_quantities(quantity, quantity, quantity, quantity);
        let zse = ZSE::new_with_quantities(quantity, quantity, quantity, quantity);
        let markets = Vec::from([Rc::clone(&smse), Rc::clone(&tase), Rc::clone(&zse)]);

        let trader_name = "TEST_TRADER".to_string();
        let strategy = MostSimpleStrategy::new(
            vec![Rc::clone(&smse), Rc::clone(&tase), Rc::clone(&zse)],
            &trader_name,
        );

        // test with very high bid
        let bid = 1_000_000.0;
        let res = strategy.find_cheapest_good(bid);
        assert_eq!(
            true,
            res.is_some(),
            "For a high bid of {} something should be found",
            bid
        );
        let (market_name, (price, good)) = res.unwrap();
        assert_ne!(
            DEFAULT_GOOD_KIND,
            good.get_kind(),
            "Found good can't be of kind {}",
            DEFAULT_GOOD_KIND
        );
        assert!(
            price <= bid,
            "Cheapest price can't be higher than bid of {}",
            bid
        );
        let cheapest_market = &markets
            .iter()
            .find(|m| m.as_ref().borrow().get_name() == market_name)
            .unwrap();
        let cheapest_price = cheapest_market
            .borrow_mut()
            .get_buy_price(good.get_kind(), good.get_qty())
            .unwrap();
        assert_eq!(
            cheapest_price, price,
            "Cheapest found price {} must be equal to the one of the market {}",
            price, cheapest_price
        );

        // test with 0.0 as bid
        let bid = 0.0;
        let res = strategy.find_cheapest_good(bid);
        assert_ne!(
            true,
            res.is_some(),
            "No good should be found for bid of {}",
            bid
        );
    }
}*/
