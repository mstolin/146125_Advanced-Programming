use crate::strategy::strategy::Strategy;
use crate::MarketRef;
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::borrow::{Borrow, BorrowMut};
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::ops::Index;
use std::rc::Rc;
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::good_label::GoodLabel;
use unitn_market_2022::market::Market;
use unitn_market_2022::wait_one_day;

type BuyTokenHistory = (String, f32, String); // (market name, bid, buy token)
type SellTokenHistory = (String, Good, String); // (market name, locked good, buy token)
type GoodHistory = (f32, Good); // (eur price, bought good with bought quantity)

pub struct MostSimpleStrategy {
    buy_tokens: RefCell<Vec<BuyTokenHistory>>,
    buy_history: RefCell<Vec<GoodHistory>>,
    sell_tokens: RefCell<Vec<SellTokenHistory>>,
    markets: Vec<MarketRef>,
}

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
        //market: &Ref<dyn Market>,
        max_eur: f32,
    ) -> Option<(f32, f32)> {
        if label.good_kind == DEFAULT_GOOD_KIND {
            // Its not smart to buy eur for eur
            return None;
        }

        let mut tried_qty = label.quantity; // start with max available quantity
        let max_tries = (tried_qty / 2.0) as u32; // todo: There has to be a better solution
        let mut tries = 0;

        while tries < max_tries {
            // get cheapest price for current quantity
            if let Ok(buy_price) = market
                .as_ref()
                .borrow()
                .get_buy_price(label.good_kind, tried_qty)
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
            .filter(|l| l.good_kind != DEFAULT_GOOD_KIND)
            .map(|label| {
                (
                    label,
                    self.find_adequate_bid(label, Rc::clone(&market), eur_qty),
                )
            })
            .filter(|(_, res)| res.is_some())
            .map(|(label, res)| {
                let (price, qty) = res.unwrap();
                (price, Good::new(label.good_kind, qty))
            })
            .reduce(|(price_a, good_a), (price_b, good_b)| {
                if good_a.get_qty() > good_b.get_qty() {
                    (price_a, good_a)
                } else {
                    (price_b, good_b)
                }
            })
    }

    /// This method tries to find the cheapest good from all markets for the given max. EUR
    /// quantity.
    /// To get the cheapest good for a single market, its uses [`find_cheapest_good_from_market()`].
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
            .reduce(|(m_a, (price_a, good_a)), (m_b, (price_b, good_b))| {
                if good_a.get_qty() > good_b.get_qty() {
                    (m_a, (price_a, good_a))
                } else {
                    (m_b, (price_b, good_b))
                }
            })
    }

    /// Returns the highest selling market + price for the bought good
    fn find_highest_selling_market_for_good<'a>(
        &'a self,
        good: &'a Good,
        buy_price: f32,
    ) -> Option<(&str, f32)> {
        self.markets
            .iter()
            .map(|m| m.as_ref().borrow())
            .map(|m| {
                (
                    m.get_name(),
                    m.get_sell_price(good.get_kind(), good.get_qty()),
                )
            })
            .filter(|(_, price)| price.is_ok())
            .map(|(name, price)| (name, price.unwrap()))
            .filter(|(_, sell_price)| *sell_price > buy_price)
            .reduce(|(market_a, price_a), (market_b, price_b)| {
                if price_a > price_b {
                    (market_a, price_a)
                } else {
                    (market_b, price_b)
                }
            })
    }

    fn increase_eur_qty(&self, goods: &mut Vec<Good>, merge_eur: Good) {
        let eur = goods.iter_mut().find(|g| g.get_kind() == DEFAULT_GOOD_KIND);
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

    fn find_market_for_name(&self, name: &String) -> Option<&MarketRef> {
        self.markets
            .iter()
            .find(|m| m.as_ref().borrow().get_name().to_string() == *name)
    }

    fn can_we_sell(&self, inventory: &Vec<Good>) -> bool {
        inventory
            .iter()
            .filter(|g| g.get_kind() != DEFAULT_GOOD_KIND)
            .count()
            > 0
    }

    fn find_highest_selling_market(&self, good: &Good, buy_price: f32) -> Option<(f32, String)> {
        for market in &self.markets {
            let market = Rc::clone(market);
            let market = market.as_ref().borrow();
            if let Ok(sell_price) = market.get_sell_price(good.get_kind(), good.get_qty()) {
                if sell_price > buy_price {
                    // sell price is bigger than our buy price
                    return Some((sell_price, market.get_name().to_string()));
                }
            }
        }
        None
    }
}

impl Strategy for MostSimpleStrategy {
    fn new(markets: Vec<MarketRef>) -> Self {
        Self {
            buy_tokens: RefCell::new(Vec::new()),
            buy_history: RefCell::new(Vec::new()),
            sell_tokens: RefCell::new(Vec::new()),
            markets,
        }
    }

    fn get_markets(&self) -> &Vec<MarketRef> {
        self.markets.borrow()
    }

    fn sell_remaining_goods(&self) {
        // TODO

    }

    fn apply(&self, goods: &mut Vec<Good>, trader_name: &String) {
        let mut buy_tokens = self.buy_tokens.borrow_mut();
        // this is our eur good (merge and split from this ref)
        let mut eur_qty = self
            .get_mut_good_for_kind(GoodKind::EUR, goods)
            .unwrap()
            .get_qty(); // todo: Maybe better error handling, but eur is always there

        // 1. Find cheapest good to buy
        let cheapest_good = self.find_cheapest_good(eur_qty);

        // 2. If a cheapest good has been found, try to lock it
        if let Some((market_name, (bid, good))) = cheapest_good {
            let market = self.find_market_for_name(&market_name).unwrap();
            let mut market = market.as_ref().borrow_mut();

            if market.get_name() == market_name {
                // 2. Lock good to buy
                let token =
                    market.lock_buy(good.get_kind(), good.get_qty(), bid, trader_name.clone());
                if let Ok(token) = token {
                    buy_tokens.push((market_name.clone(), bid, token.clone()));
                }
            }
        }

        // 3. Buy every locked good
        let mut bought_tokens: Vec<BuyTokenHistory> = Vec::new();
        for buy_token_history in buy_tokens.iter_mut() {
            let (market_name, bid, token) = buy_token_history;
            let market = self.find_market_for_name(market_name).unwrap();
            let mut market = market.as_ref().borrow_mut();

            //let mut cash = Good::new(GoodKind::EUR, *bid);
            let mut our_eur = self.get_mut_good_for_kind(GoodKind::EUR, goods).unwrap();
            if let Ok(bought_good) = market.buy(token.clone(), &mut our_eur) {
                // todo Better error handling
                let mut our_good = self
                    .get_mut_good_for_kind(bought_good.get_kind(), goods)
                    .unwrap();
                let _ = our_good.merge(bought_good.clone()); // TODO Check this: like in sell
                                                             // reduce our EURs with the buy price
                // also add the buy to the buy history
                self.buy_history.borrow_mut().push((*bid, bought_good));
                bought_tokens.push(buy_token_history.clone());
            }
        }

        // remove buy token
        for buy_token_history in bought_tokens {
            let index = buy_tokens
                .iter()
                .position(|t| *t == buy_token_history)
                .unwrap();
            buy_tokens.remove(index);
        }

        // ## SELL

        // 1. Lock
        for (buy_price, good) in self.buy_history.borrow().iter() {
            // Do not sell EUR
            if good.get_kind() == DEFAULT_GOOD_KIND {
                continue;
            }

            // Find the market with highest sell price
            if let Some((sell_price, market_name)) =
                self.find_highest_selling_market(good, *buy_price)
            {
                // found an adequate market, now need to lock the good
                let market = self.find_market_for_name(&market_name).unwrap(); // todo error handling
                let mut market = market.as_ref().borrow_mut();
                if let Ok(token) = market.lock_sell(
                    good.get_kind(),
                    good.get_qty(),
                    sell_price,
                    trader_name.clone(),
                ) {
                    // successfully locked good for sell, add token with price to the sell token history
                    self.sell_tokens
                        .borrow_mut()
                        .push((market_name, good.clone(), token));
                }
            }
        }

        // 2. Actually sell
        let mut sold_tokens: Vec<SellTokenHistory> = Vec::new();
        let mut sell_tokens = self.sell_tokens.borrow_mut();
        for sell_token_history in sell_tokens.iter_mut() {
            let (market_name, good, token) = sell_token_history;
            let market = self.find_market_for_name(&market_name).unwrap(); // todo error handling
            let mut market = market.as_ref().borrow_mut();

            if let Ok(cash) = market.sell(token.clone(), good) {
                let mut our_eur = self.get_mut_good_for_kind(GoodKind::EUR, goods).unwrap();
                let _ = our_eur.merge(cash); // todo handle the error
                sold_tokens.push(sell_token_history.clone());
            }
        }

        // Remove old tokens that have been bought
        for sell_token_history in sold_tokens {
            let index = sell_tokens
                .iter()
                .position(|t| *t == sell_token_history)
                .unwrap();
            sell_tokens.remove(index);
        }
    }
}

#[cfg(test)]
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
        let strategy = MostSimpleStrategy::new(markets);

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
        let strategy = MostSimpleStrategy::new(markets);
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

        let strategy = MostSimpleStrategy::new(vec![Rc::clone(&zse)]);
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

        let strategy =
            MostSimpleStrategy::new(vec![Rc::clone(&smse), Rc::clone(&tase), Rc::clone(&zse)]);

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
}
