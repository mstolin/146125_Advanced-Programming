use crate::strategy::strategy::Strategy;
use crate::MarketRef;
use log::{info, warn};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::borrow::{Borrow, BorrowMut};
use std::cell::{Cell, Ref, RefCell, RefMut};
use std::collections::{HashMap, HashSet};
use std::ops::Index;
use std::rc::Rc;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::good_label::GoodLabel;
use unitn_market_2022::market::{LockSellError, Market};
use unitn_market_2022::wait_one_day;

/// (market name, bid, buy token)
type BuyTokenHistory = (String, Payment, String);
/// (market name, locked good, sell token)
type SellTokenHistory = (String, GoodKind, String);
/// kind: [(price, quantity)]
type BuyHistory = HashMap<GoodKind, Vec<(f32, f32)>>;
type MarketOffers = HashMap<String, Vec<Payment>>;

#[derive(Clone, Debug)]
struct Payment {
    price: f32,
    quantity: f32,
    good_kind: GoodKind,
    market_name: String,
}

impl Payment {
    fn new(price: f32, quantity: f32, good_kind: GoodKind, market_name: String) -> Self {
        Self {
            price,
            quantity,
            good_kind,
            market_name,
        }
    }
}

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
    /// Number of buy operations
    buy_count: RefCell<u32>,
    /// Number of sell operations
    sell_count: RefCell<u32>,
    /// Maximum allowed difference between sell and buy operations
    max_diff_count_operations: u32,
}

/// Buying methods
impl MostSimpleStrategy {
    fn allowed_to_buy(&self) -> bool {
        let sell_count = *self.sell_count.borrow();
        let buy_count = *self.buy_count.borrow();
        let diff = sell_count.abs_diff(buy_count);
        diff <= self.max_diff_count_operations
    }

    /// Returns an adequate bid for the given EUR quantity.
    /// This method tries to get the maximum quantity of the good for the given label,
    /// that this strategy can buy with the given amount of EUR.
    /// It is possible that the given EUR is too low and no quantity will be found.
    ///
    /// The return value is (buy price in EUR, quantity of good).
    fn find_adequate_bid(&self, market: MarketRef, max_eur: f32, kind: &GoodKind) -> Option<Payment> {
        if *kind == GoodKind::EUR || max_eur <= 0.0 {
            // Its not smart to buy eur for eur
            return None;
        }

        let market = market.as_ref().borrow();

        // start with max available quantity
        let mut market_goods = market.get_goods();
        let (mut tried_qty, market_ex_rate) = market_goods
            .iter()
            .find(|g| g.good_kind == *kind)
            .map(|g| (g.quantity, g.exchange_rate_buy))
            .unwrap_or_default();
        let max_tries = (tried_qty / 2.0) as u32; // todo: There has to be a better solution
        let mut tries = 0;

        while tries < max_tries {
            let buy_price = market.get_buy_price(*kind, tried_qty);
            if let Ok(buy_price) = buy_price {
                let bid_ex_rate = buy_price / tried_qty;

                if buy_price <= max_eur && bid_ex_rate < (market_ex_rate * 1.5) {
                    // buy price is below our maximum bid
                    let market_name = market.get_name().to_string();
                    return Some(Payment::new(buy_price, tried_qty, *kind, market_name));
                }
            }

            // reduce the qty if no adequate price was found
            let s = tried_qty / 2.0; // to go below zero, this has to be higher than the half
            tried_qty = tried_qty - s; // todo check for a more fine grained solution
            tries += 1;
        }
        None
    }

    fn find_adequate_bids<P>(
        &self,
        good_kind: &GoodKind,
        max_eur: f32,
        find_adequate_bid: P,
    ) -> HashMap<String, Payment>
    where
        P: Fn(MarketRef, f32, &GoodKind) -> Option<Payment>,
    {
        let mut bids = HashMap::new();

        if *good_kind == GoodKind::EUR || max_eur <= 0.0 {
            return bids;
        }

        for market in self.markets.iter() {
            let adequate_bid = find_adequate_bid(Rc::clone(market), max_eur, good_kind);
            if let Some(bid) = adequate_bid {
                let market = market.as_ref().borrow();
                let market_name = market.get_name().to_string();

                info!(
                    "Found adequate bid {} {} for {} EUR at {}",
                    bid.quantity, bid.good_kind, bid.price, market_name
                );

                bids.insert(market_name, bid);
            } else {
                warn!("Didn't found an adequate bid for {}", good_kind);
            }
        }
        bids
    }

    fn filter_cheapest_bid(&self, bids: &HashMap<String, Payment>) -> Option<(String, Payment)> {
        let mut cheapest_bid: Option<(String, Payment)> = None;
        for (market_name, bid) in bids.iter() {
            if let Some((cheapest_market, cheapest_bid)) = &mut cheapest_bid {
                if bid.price > cheapest_bid.price {
                    // Found a cheaper bid
                    *cheapest_market = market_name.clone();
                    *cheapest_bid = bid.clone();
                }
            } else {
                cheapest_bid = Some((market_name.clone(), bid.clone()));
            }
        }
        cheapest_bid
    }

    fn find_good_to_lock_buy(&self, inventory: &Vec<Good>) -> GoodKind {
        // shuffle the inventory first, maybe all good are empty
        let mut shuffled_inventory = inventory.clone();
        shuffled_inventory.shuffle(&mut thread_rng());

        shuffled_inventory
            .iter()
            .filter(|g| g.get_kind() != GoodKind::EUR)
            .reduce(|a, b| if a.get_qty() < b.get_qty() { a } else { b })
            .map(|g| g.get_kind())
            // we can unwrap, because the good does exist
            .unwrap()
    }

    fn lock_bid(&self, market_name: &String, bid: &Payment) {
        // We can be sure the market exist
        let market = self.find_market_for_name(market_name).unwrap();
        let mut market = market.as_ref().borrow_mut();

        if market.get_name() == market_name {
            // 2. Lock good to buy
            let token = market.lock_buy(
                bid.good_kind,
                bid.quantity,
                bid.price,
                self.trader_name.clone(),
            );
            if let Ok(token) = token {
                info!(
                    "Locked for buy: good {} {} for {} EUR at market {}",
                    bid.quantity, bid.good_kind, bid.price, market_name
                );
                let mut buy_tokens = self.buy_tokens.borrow_mut();
                buy_tokens.push((market_name.clone(), bid.clone(), token.clone()));
            } else {
                warn!("Not able to lock good for buy: {:?}", token);
            }
        }
    }

    // todo: Rename lock_bids
    fn lock_cheapest_bid(&self, inventory: &Vec<Good>) {
        // 1. Find good kind to buy
        let kind_to_buy = self.find_good_to_lock_buy(inventory);
        // 2. Find adequate bids per market
        let eur_qty = self
            .get_good_for_kind(GoodKind::EUR, inventory)
            .unwrap()
            .get_qty();
        let adequate_bids =
            self.find_adequate_bids(&kind_to_buy, eur_qty * 0.3, |market, max_eur, kind| {
                self.find_adequate_bid(market, max_eur, kind)
            });
        // 3. Find cheapest bid among adequate bids for kind
        let cheapest_bids = self.filter_cheapest_bid(&adequate_bids);
        // 4. Lock cheapest bid
        if let Some((market, bid)) = cheapest_bids {
            info!(
                "Found an adequate bid: {} {} for {} EUR at {}",
                bid.good_kind, bid.quantity, bid.price, market
            );
            self.lock_bid(&market, &bid);
        }
    }

    fn buy_locked_goods(&self, inventory: &mut Vec<Good>) {
        if !self.allowed_to_buy() {
            warn!("Not allowed to buy");
            return;
        }

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
            let bought_good = market.buy(token.clone(), eur);
            if let Ok(bought_good) = bought_good {
                info!(
                    "Bought good {} {} for {} EUR at market {}",
                    bought_good.get_qty(),
                    bought_good.get_kind(),
                    bid.price,
                    market_name
                );
                self.add_to_buy_history(&bought_good, bid.price);
                // todo Better error handling
                let mut our_good = self
                    .get_mut_good_for_kind(bought_good.get_kind(), inventory)
                    .unwrap();
                let _ = our_good.merge(bought_good.clone());
                // todo: Why push, just do remove_buy_token(&token)??
                bought_tokens.push(token.clone());
                // Increase buy count
                let mut buy_count = self.buy_count.borrow_mut();
                *buy_count += 1;
            } else {
                warn!("Could not buy good: {:?}", bought_good);
            }
        }
    }

    /// This method adds the average price of the current buy to the history.
    /// Call this method after a successful buy.
    fn add_to_buy_history(&self, bought_good: &Good, bid: f32) {
        let mut buy_history = self.buy_history.borrow_mut();
        if let Some(mut kind_history) = buy_history.get_mut(&bought_good.get_kind()) {
            kind_history.push((bid, bought_good.get_qty()))
        } else {
            let kind_history = vec![(bid, bought_good.get_qty())];
            buy_history.insert(bought_good.get_kind(), kind_history);
        }
    }

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
    /// Returns (offer, quantity)
    fn find_adequate_offer(&self, market: MarketRef, good: &Good) -> Option<Payment> {
        if good.get_kind() == GoodKind::EUR || good.get_qty() <= 0.0 {
            return None;
        }

        let market = market.as_ref().borrow();
        let average_buy_price = self.get_average_price_for_good(&good.get_kind());

        // By default, start with max quantity available
        let mut quantity = good.get_qty();
        let max_tries = 40; //(quantity / 2.0) as u32; // todo: There has to be a better solution
        let mut tries: u32 = 0;

        while tries < max_tries {
            let sell_price = market.get_sell_price(good.get_kind(), quantity);
            if let Ok(sell_price) = sell_price {
                let avg = sell_price / quantity;
                // try find an avg. sell price that is higher than our avg. buy price to make profit
                if avg > average_buy_price {
                    let market_name = market.get_name().to_string();
                    return Some(Payment::new(sell_price, quantity, good.get_kind(), market_name));
                }
            } else {
                warn!(
                    "Error by trying to find an adequate offer: {:?}",
                    sell_price
                );
            }

            // no good price for current quantity has been found, so lower the quantity to try with
            let s = quantity / 2.0;
            quantity = quantity - s; // todo check for a more fine grained solution
            tries += 1;
        }

        None
    }

    fn find_offers_for_markets<P>(
        &self,
        inventory: &Vec<Good>,
        find_adequate_offer: P,
    ) -> MarketOffers
    where
        P: Fn(MarketRef, &Good) -> Option<Payment>,
    {
        let mut offers = HashMap::new();

        for market in self.markets.iter() {
            let mut market_offers = Vec::new();

            for good in inventory.iter() {
                if good.get_kind() == GoodKind::EUR || good.get_qty() <= 0.0 {
                    // we don't want to sell EUR
                    continue;
                }

                let offer = find_adequate_offer(Rc::clone(market), good);

                let market = market.as_ref().borrow();
                let market_name = market.get_name().to_string();

                if let Some(offer) = offer {
                    info!(
                        "Found an adequate offer of{} {} for {} EUR at market {}",
                        offer.quantity, offer.good_kind, offer.price, market_name
                    );
                    market_offers.push(offer);
                } else {
                    warn!("Didn't found an adequate offer for good ({:?})", good);
                }
            }

            let market_name = market.as_ref().borrow().get_name().to_string();
            if !market_offers.is_empty() {
                offers.insert(market_name.clone(), market_offers);
            }
        }

        offers
    }

    fn filter_best_offers(&self, offers: &MarketOffers) -> HashMap<GoodKind, (String, Payment)> {
        let mut best_offers: HashMap<GoodKind, (String, Payment)> = HashMap::new();
        for (market_name, market_offers) in offers.iter() {
            // try to find the best offer for the current good
            for offer in market_offers.iter() {
                // Does a best offer already exist?
                if let Some((best_market, best_offer)) = best_offers.get_mut(&offer.good_kind) {
                    // Some best offer already exists, so compare it
                    if offer.price > best_offer.price {
                        // found a new best price => update
                        *best_market = market_name.clone();
                        *best_offer = offer.clone();
                    }
                } else {
                    // has no offer yet, so insert current offer
                    best_offers.insert(
                        offer.good_kind.clone(),
                        (market_name.clone(), offer.clone()),
                    );
                }
            }
        }
        best_offers
    }

    fn lock_offer(&self, mut market: RefMut<dyn Market>, offer: Payment, is_second_try: bool) {
        // try to lock it
        let token = market.lock_sell(
            offer.good_kind,
            offer.quantity,
            offer.price,
            self.trader_name.clone(),
        );
        let market_name = market.get_name().to_string();

        match token {
            Ok(token) => {
                // lock was successful, save token
                info!(
                    "Locked good for sell {} {} for offer {} EUR at market {}",
                    offer.quantity, offer.good_kind, offer.price, market_name
                );
                self.sell_tokens
                    .borrow_mut()
                    .push((market_name, offer.good_kind, token)); // TODO: Make custom struct Offer { }
            }
            Err(err) => match err {
                LockSellError::OfferTooHigh {
                    offered_good_kind,
                    offered_good_quantity,
                    high_offer,
                    highest_acceptable_offer,
                } => {
                    warn!("(Lock for sell) Offer too high, try again. ({:?})", err);
                    // Check if highest acceptable offer is adequate and lock
                    let avg = highest_acceptable_offer / offered_good_quantity;
                    let adequate_avg = self.get_average_price_for_good(&offered_good_kind);
                    if avg > adequate_avg && !is_second_try {
                        let offer =
                            Payment::new(highest_acceptable_offer, offer.quantity, offer.good_kind, market_name);
                        self.lock_offer(market, offer, true);
                    }
                }
                _ => warn!("Could not lock good for sell: {:?}", err),
            },
        }
    }

    fn lock_offers(&self, offers: &HashMap<GoodKind, (String, Payment)>) {
        for (kind, (market_name, offer)) in offers.iter() {
            // We can be sure, this market exist
            let market = self
                .markets
                .iter()
                .find(|m| m.as_ref().borrow().get_name().to_string() == *market_name)
                .map(|m| Rc::clone(m))
                .unwrap(); // todo Update the find_market method
            let mut market = market.as_ref().borrow_mut();
            self.lock_offer(market, offer.clone(), false);
        }
    }

    fn lock_goods_for_sell(&self, inventory: &mut Vec<Good>) {
        // 1. Find the quantity we can sell with the highest profit for that market for every good
        let offers = self.find_offers_for_markets(inventory, |m, g| self.find_adequate_offer(m, g));
        // 2. Find the best offer for every good
        let best_offers = self.filter_best_offers(&offers);
        // 3. Lock best offers
        self.lock_offers(&best_offers);
        // 4. Remove all offers we can't sell anymore and repeat
        // todo: Is this necessary?
    }

    fn sell_locked_goods(&self, inventory: &mut Vec<Good>) {
        let mut sold_tokens = self.sold_tokens.borrow_mut();
        let mut sell_tokens = self.sell_tokens.borrow_mut();

        // loop over all sell tokens and sell the good
        for (market_name, good_kind, token) in sell_tokens.iter_mut() {
            // We can be sure that this market exist
            let market = self.find_market_for_name(&market_name).unwrap();
            let mut market = market.as_ref().borrow_mut();

            let mut good = self
                .get_mut_good_for_kind(good_kind.clone(), inventory)
                .unwrap();
            let old_quantity = good.get_qty();
            let cash = market.sell(token.clone(), good);
            if let Ok(cash) = cash {
                let new_quantity = old_quantity - good.get_qty();
                info!(
                    "Sold {} {} for {} EUR at market {}",
                    new_quantity,
                    good.get_kind(),
                    cash.get_qty(),
                    market_name
                );
                // add (remove) from history
                self.add_to_buy_history(
                    &Good::new(good_kind.clone(), new_quantity * (-1.0)),
                    (cash.get_qty() * (-1.0)),
                );
                println!("{:?}", self.buy_history);
                // Now increase our eur quantity
                let mut eur = self
                    .get_mut_good_for_kind(GoodKind::EUR, inventory)
                    .unwrap();
                let _ = eur.merge(cash); // todo handle the error
                sold_tokens.push(token.clone());
                // Increase sell count
                let mut sell_count = self.sell_count.borrow_mut();
                *sell_count += 1;
            } else {
                warn!("Could not sold {}: {:?}", good.get_kind(), cash);
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

    // todo: Make kind ref
    fn get_mut_good_for_kind<'a>(
        &'a self,
        kind: GoodKind,
        inventory: &'a mut Vec<Good>,
    ) -> Option<&mut Good> {
        inventory.iter_mut().find(|g| g.get_kind() == kind)
    }

    // todo: Make kind ref
    fn get_good_for_kind<'a>(&'a self, kind: GoodKind, inventory: &'a Vec<Good>) -> Option<&Good> {
        inventory.iter().find(|g| g.get_kind() == kind)
    }

    fn find_market_for_name(&self, name: &String) -> Option<&MarketRef> {
        self.markets
            .iter()
            .find(|m| m.as_ref().borrow().get_name().to_string() == *name)
    }

    // todo: get_avg_buy_price_per_piece
    fn get_average_price_for_good(&self, kind: &GoodKind) -> f32 {
        let buy_history = self.buy_history.borrow();
        if let Some(good_history) = buy_history.get(kind) {
            if !good_history.is_empty() {
                let overall_sum_paid = good_history
                    .iter()
                    .map(|(bid, _)| *bid)
                    .reduce(|bid_a, bid_b| bid_a + bid_b)
                    .unwrap_or_default();
                let overall_quantity_bought = good_history
                    .iter()
                    .map(|(_, quantity)| *quantity)
                    .reduce(|qty_a, qty_b| qty_a + qty_b)
                    .unwrap_or_default();
                return overall_sum_paid / overall_quantity_bought;
            }
        }
        0.0
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
            sell_count: RefCell::new(0),
            buy_count: RefCell::new(0),
            max_diff_count_operations: 5,
        }
    }

    fn get_markets(&self) -> &Vec<MarketRef> {
        self.markets.borrow()
    }

    fn sell_remaining_goods(&self, goods: &mut Vec<Good>) {
        info!("-------------------------");
        // Try to sell everything we have for the best price possible
        let offers = self.find_offers_for_markets(goods, |market, good| {
            let market = market.as_ref().borrow();
            // Just return the offer for the max quantity
            if let Ok(price) = market.get_sell_price(good.get_kind(), good.get_qty()) {
                let market_name = market.get_name().to_string();
                Some(Payment::new(price, good.get_qty(), good.get_kind(), market_name))
            } else {
                // todo: OfferTooHight -> Just return the highest acceptable offer
                None
            }
        });
        let best_offers = self.filter_best_offers(&offers);
        self.lock_offers(&best_offers);
        self.sell_locked_goods(goods);
    }

    fn apply(&self, goods: &mut Vec<Good>) {
        self.lock_cheapest_bid(goods); // 1. Lock buy the cheapest good we can find
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
