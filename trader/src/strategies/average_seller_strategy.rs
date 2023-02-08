//! This strategy tries to buy the cheapest goods available, and the sell
//! them, for them for a higher price than the average buy price per piece.
//!
//! # Buying Strategy
//!
//! This strategy always tries to buy the cheapest good available. The problems that arise with
//! that strategy is:
//!
//! 1. What good do we buy
//! 2. At what max. price to we buy
//! 3. What quantity do we buy
//! 4. How do we stop the trader to buy (spent all the available EUR)
//!
//! The selection of the good to buy is simple: Just select the good with the lowest owned
//! quantity. The assumptions are, if the quantity is low, then markets own a lot of that good
//! and price is cheap.
//!
//! To solve the second problem, the strategy is allowed to pay at max. 30% of the owned EUR
//! quantity. 30% because there are 3 different goods to buy.
//!
//! For the second problem, the strategy tries to find the highest quantity for the max. price.
//!
//! To stop the trader to spent all EUR, the strategy has a specific threshold of allowed buy
//! operations. This threshold depends on the sell operations. The difference between a buy
//! and a sell operations is not allowed to be higher than *n* (e.g. 5). If the trader has
//! performed *n* more buy operations than sell operations, the trader is not allowed to buy,
//! and it is expected that the trader sells before buying again.
//!
//! # Selling Strategy
//!
//! The strategy for selling is simple: Just sell at a higher price than bought. To do that, it
//! calculates the average price for one piece of the good paid by now and compares that with the
//! sell price for one a single piece given by market. If found, sell as much as possible.
use crate::strategies::strategy::Strategy;
use crate::MarketRef;
use log::{info, warn};
use rand::seq::SliceRandom;
use rand::thread_rng;
use std::borrow::Borrow;
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;

use std::rc::Rc;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;

use unitn_market_2022::market::{LockSellError, Market};

/// This type represents the history for either buy or sell tokens.
/// Each token has a corresponding offer or bid (as instance of [`Payment`]).
type TokenHistory = (String, Payment);
/// This type represents the buy history; { [`GoodKind`]: (buy_price, bought_quantity) }
type BuyHistory = HashMap<GoodKind, Vec<(f32, f32)>>;

#[derive(Clone, Debug)]
/// A `Payment` instance either represents an offer of a market or a bid
/// from the trader for a good.
struct Payment {
    /// The offer or bid
    price: f32,
    /// Quantity to sell or buy
    quantity: f32,
    /// [`GoodKind`] of the good this payment is about
    good_kind: GoodKind,
    /// The market that accepted/created this payment
    market_name: String,
}

impl Payment {
    /// Constructs a new [`Payment`] instance
    fn new(price: f32, quantity: f32, good_kind: GoodKind, market_name: String) -> Self {
        Self {
            price,
            quantity,
            good_kind,
            market_name,
        }
    }
}

/// The implementation of the `AverageSellerStrategy`.
pub struct AverageSellerStrategy {
    /// Name of the trader using this strategy
    trader_name: String,
    /// All markets this strategy works with
    markets: Vec<MarketRef>,
    /// Storage for buy tokens
    buy_tokens: RefCell<Vec<TokenHistory>>,
    /// Storage for tokens that have been bought
    bought_tokens: RefCell<Vec<String>>,
    /// History of bought goods
    buy_history: RefCell<BuyHistory>,
    /// Storage for sell tokens
    sell_tokens: RefCell<Vec<TokenHistory>>,
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
impl AverageSellerStrategy {
    /// Returns a boolean that represents if a buy operation is allowed at the moment.
    /// A buy operation may be disallowed, if the absolute difference between the number of
    /// sell and buy operations is lower than the number defined as `max_diff_count_operations`.
    ///
    /// For example:
    /// `max_diff_count_operations` = 5, `buy_operations` = 4, `sell_operations` = 2.
    /// Then, the difference is 2 (< allowed diff) so a buy is allowed.
    ///
    /// This is done to prevent the trader (using this strategy) to buy all time, or in other words
    /// to spent all the owned money.
    fn allowed_to_buy(&self) -> bool {
        let sell_count = *self.sell_count.borrow();
        let buy_count = *self.buy_count.borrow();
        let diff = sell_count.abs_diff(buy_count);
        diff <= self.max_diff_count_operations
    }

    /// Returns an adequate bid for the wanted good and the max. available EUR quantity.
    /// In this context, adequate means *"a good deal"*, that price of the bid is
    /// lower than the max. money available, and the receiving quantity is high enough to be
    /// considered good.
    ///
    /// It tries to find a quantity until the price for that quantity is below the given max. eur
    /// threshold. It is possible, that no adequate bid will be found.
    fn find_adequate_bid(
        &self,
        market: MarketRef,
        max_eur: f32,
        kind: &GoodKind,
    ) -> Option<Payment> {
        if *kind == GoodKind::EUR || max_eur <= 0.0 {
            // Its not smart to buy eur for eur
            return None;
        }

        let market = market.as_ref().borrow();

        // start with max available quantity
        let market_goods = market.get_goods();
        let mut tried_qty = market_goods
            .iter()
            .find(|g| g.good_kind == *kind)
            .map(|g| g.quantity)
            .unwrap_or_default();
        let max_tries = (tried_qty / 2.0) as u32; // todo: There has to be a better solution
        let mut tries = 0;

        while tries < max_tries {
            let buy_price = market.get_buy_price(*kind, tried_qty);
            if let Ok(buy_price) = buy_price {
                if buy_price.is_subnormal() {
                    return None;
                }

                if buy_price <= max_eur {
                    // buy price is below our maximum bid
                    let market_name = market.get_name().to_string();
                    return Some(Payment::new(buy_price, tried_qty, *kind, market_name));
                }
            }

            // reduce the qty if no adequate price was found
            let s = tried_qty / 2.0; // to go below zero, this has to be higher than the half
            tried_qty -= s; // todo check for a more fine grained solution
            tries += 1;
        }
        None
    }

    /// This method tries to find an adequate bid for every given market. It requires a predicate
    /// function as parameter. By default, this should be [`AverageSellerStrategy::find_adequate_bid`].
    fn find_adequate_bids<P>(
        &self,
        good_kind: &GoodKind,
        max_eur: f32,
        find_adequate_bid: P,
    ) -> Vec<Payment>
    where
        P: Fn(MarketRef, f32, &GoodKind) -> Option<Payment>,
    {
        let mut bids = Vec::new();

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

                bids.push(bid);
            } else {
                warn!("Didn't found an adequate bid for {}", good_kind);
            }
        }
        bids
    }

    /// This method filters the best bid among all given bids.
    /// The best bid is considered the one, that is the cheapest.
    fn filter_cheapest_bid(&self, bids: &[Payment]) -> Option<Payment> {
        let mut cheapest_bid: Option<Payment> = None;
        for bid in bids.iter() {
            if let Some(cheapest_bid) = &mut cheapest_bid {
                if bid.price > cheapest_bid.price {
                    // Found a cheaper bid
                    *cheapest_bid = bid.clone();
                }
            } else {
                cheapest_bid = Some(bid.clone());
            }
        }
        cheapest_bid
    }

    /// This method find the best goo to lock buy. In this case, *best* is considered the good
    /// where this trader owns the lowest quantity.
    /// This is based on the assumption, that the quantity that hasn't been bought much, will
    /// probably be the cheapest.
    fn find_good_to_lock_buy(&self, inventory: &[Good]) -> GoodKind {
        // shuffle the inventory first, maybe all good are empty
        let mut shuffled_inventory = inventory.to_owned();
        shuffled_inventory.shuffle(&mut thread_rng());

        shuffled_inventory
            .iter()
            .filter(|g| g.get_kind() != GoodKind::EUR)
            .reduce(|a, b| if a.get_qty() < b.get_qty() { a } else { b })
            .map(|g| g.get_kind())
            // we can unwrap, because the good does exist
            .unwrap()
    }

    /// This method locks the given bid for buy.
    fn lock_bid(&self, bid: &Payment) {
        // We can be sure the market exist
        let market_name = &bid.market_name;
        let market = self.find_market_for_name(market_name);

        if let Some(market) = market {
            let mut market = market.as_ref().borrow_mut();
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
                buy_tokens.push((token, bid.clone()));
            } else {
                warn!("Not able to lock good for buy: {:?}", token);
            }
        }
    }

    /// This method tries to lock all bids.
    fn lock_bids(&self, inventory: &[Good]) {
        // 1. Find good kind to buy
        let kind_to_buy = self.find_good_to_lock_buy(inventory);
        // 2. Find adequate bids per market
        let eur_qty = self
            .get_good_for_kind(&GoodKind::EUR, inventory)
            .unwrap()
            .get_qty();
        let adequate_bids =
            self.find_adequate_bids(&kind_to_buy, eur_qty * 0.3, |market, max_eur, kind| {
                self.find_adequate_bid(market, max_eur, kind)
            });
        // 3. Find cheapest bid among adequate bids for kind
        let cheapest_bids = self.filter_cheapest_bid(&adequate_bids);
        // 4. Lock cheapest bid
        if let Some(bid) = cheapest_bids {
            info!(
                "Found an adequate bid: {} {} for {} EUR at {}",
                bid.good_kind, bid.quantity, bid.price, bid.market_name
            );
            self.lock_bid(&bid);
        }
    }

    /// This methods tries to buy all goods that have been locked in `buy_tokens`.
    /// If a buy wasn't successful, it will retry a second time with an updated price.
    /// The updated price is usually received by the error message.
    /// After the buy was successful, the bid is added to the `buy_history`.
    fn buy_locked_goods(&self, inventory: &mut [Good]) {
        if !self.allowed_to_buy() {
            warn!("Not allowed to buy");
            return;
        }

        let buy_tokens = self.buy_tokens.borrow_mut();
        let mut bought_tokens = self.bought_tokens.borrow_mut();

        // loop over all buy tokens and buy the goods
        for (token, bid) in buy_tokens.iter() {
            // borrow market as mut
            let market = self.find_market_for_name(&bid.market_name).unwrap();
            let mut market = market.as_ref().borrow_mut();

            let eur = self
                .get_mut_good_for_kind(&GoodKind::EUR, inventory)
                .unwrap();
            let bought_good = market.buy(token.clone(), eur);
            if let Ok(bought_good) = bought_good {
                info!(
                    "Bought good {} {} for {} EUR at market {}",
                    bought_good.get_qty(),
                    bought_good.get_kind(),
                    bid.price,
                    bid.market_name
                );
                self.add_to_buy_history(&bought_good, bid.price);
                let our_good = self
                    .get_mut_good_for_kind(&bought_good.get_kind(), inventory)
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

    /// This method adds the bought good and the payed price to the `buy_history`.
    /// Call this method after a successful buy.
    fn add_to_buy_history(&self, bought_good: &Good, bid: f32) {
        let mut buy_history = self.buy_history.borrow_mut();
        if let Some(kind_history) = buy_history.get_mut(&bought_good.get_kind()) {
            kind_history.push((bid, bought_good.get_qty()))
        } else {
            let kind_history = vec![(bid, bought_good.get_qty())];
            buy_history.insert(bought_good.get_kind(), kind_history);
        }
    }

    /// This clears all token that have been bought. Bought tokens are saved in `bought_tokens`.
    /// If `buy_tokens` contains the same token, it will be removed.
    fn clear_bought_tokens(&self) {
        let mut buy_tokens = self.buy_tokens.borrow_mut();
        let bought_tokens = self.bought_tokens.borrow();

        for bought_token in bought_tokens.iter() {
            if let Some(index) = buy_tokens
                .iter()
                .position(|(token, _)| *token == *bought_token)
            {
                // token was found -> remove it
                buy_tokens.remove(index);
            }
        }

        // todo: Clear bought tokens ??
    }
}

/// Selling Methods
impl AverageSellerStrategy {
    /// Tries to find an adequate offer for given good (a good that this trader has bought) and at
    /// the wanted market.
    /// An offer is considered adequate, when the price per piece is below than the price per piece
    /// this trader has paid.
    /// This method tries to find an adequate offer for every possible quantity. It is possible,
    /// that no adequate offer will be found for any quantity.
    fn find_adequate_offer(&self, market: MarketRef, good: &Good) -> Option<Payment> {
        if good.get_kind() == GoodKind::EUR || good.get_qty() <= 0.0 {
            return None;
        }

        let market = market.as_ref().borrow();
        let average_buy_price = self.get_avg_buy_price_per_piece(&good.get_kind());

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
                    return Some(Payment::new(
                        sell_price,
                        quantity,
                        good.get_kind(),
                        market_name,
                    ));
                }
            } else {
                warn!(
                    "Error by trying to find an adequate offer: {:?}",
                    sell_price
                );
            }

            // no good price for current quantity has been found, so lower the quantity to try with
            let s = quantity / 2.0;
            quantity -= s; // todo check for a more fine grained solution
            tries += 1;
        }

        None
    }

    /// This method tries to find adequate offers for all given markets.
    /// As parameter, it takes a function that is being executed to find an adequate offer for a
    /// specific market. By default this is [`AverageSellerStrategy::find_adequate_offer`].
    fn find_offers_for_markets<P>(&self, inventory: &[Good], find_adequate_offer: P) -> Vec<Payment>
    where
        P: Fn(MarketRef, &Good) -> Option<Payment>,
    {
        let mut offers = Vec::new();

        for market in self.markets.iter() {
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
                    offers.push(offer);
                } else {
                    warn!("Didn't found an adequate offer for good ({:?})", good);
                }
            }
        }

        offers
    }

    /// This method filters the best offers from the given offers. A best offer is the one, where
    /// the trader makes the most profit from. Therefore, if the sell price for an offer (for a
    /// specific kind) is higher than another, then it will choose the one with the highest
    /// sell price.
    fn filter_best_offers(&self, offers: &[Payment]) -> Vec<Payment> {
        let mut best_offers: Vec<Payment> = Vec::new();
        for offer in offers.iter() {
            // try to find the best offer for the current good
            let best_offer = best_offers
                .iter_mut()
                .find(|p| p.good_kind == offer.good_kind);
            if let Some(best_offer) = best_offer {
                // Some best offer already exists, so compare it
                if offer.price > best_offer.price {
                    // found a new best price => update
                    *best_offer = offer.clone();
                }
            } else {
                // has no offer yet, so insert current offer
                best_offers.push(offer.clone());
            }
        }
        best_offers
    }

    /// This method locks an offer at the given market. If there is an error during the lock
    /// operation, the method will try a second time with an updated offer. At second try, it checks
    /// if the updated offer is still adequate, if yes, it locks again. The updated price is
    /// received from the error message.
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
                self.sell_tokens.borrow_mut().push((token, offer));
            }
            Err(err) => match err {
                LockSellError::OfferTooHigh {
                    offered_good_kind,
                    offered_good_quantity,
                    high_offer: _,
                    highest_acceptable_offer,
                } => {
                    warn!("(Lock for sell) Offer too high, try again. ({:?})", err);
                    // Check if highest acceptable offer is adequate and lock
                    let avg = highest_acceptable_offer / offered_good_quantity;
                    let adequate_avg = self.get_avg_buy_price_per_piece(&offered_good_kind);
                    if avg > adequate_avg && !is_second_try {
                        let offer = Payment::new(
                            highest_acceptable_offer,
                            offer.quantity,
                            offer.good_kind,
                            market_name,
                        );
                        self.lock_offer(market, offer, true);
                    }
                }
                _ => warn!("Could not lock good for sell: {:?}", err),
            },
        }
    }

    /// This method tries to lock all given offers.
    fn lock_offers(&self, offers: &[Payment]) {
        for offer in offers.iter() {
            // We can be sure, this market exist
            let market = self
                .markets
                .iter()
                .find(|m| *m.as_ref().borrow().get_name() == offer.market_name)
                .map(Rc::clone)
                .unwrap(); // todo Update the find_market method
            let market = market.as_ref().borrow_mut();
            self.lock_offer(market, offer.clone(), false);
        }
    }

    /// This method first tries to find adequate offers to sell, and then tries to lock those
    /// offers.
    fn lock_goods_for_sell(&self, inventory: &mut [Good]) {
        // 1. Find the quantity we can sell with the highest profit for that market for every good
        let offers = self.find_offers_for_markets(inventory, |m, g| self.find_adequate_offer(m, g));
        // 2. Find the best offer for every good
        let best_offers = self.filter_best_offers(&offers);
        // 3. Lock best offers
        self.lock_offers(&best_offers);
        // 4. Remove all offers we can't sell anymore and repeat
        // todo: Is this necessary?
    }

    /// This method tries to sell all locked goods, where a token is found in `sell_tokens`.
    /// After a successful sell, it increases the trader EUR quantity and adds the offer (as
    /// negative numbers) to the buy history.
    fn sell_locked_goods(&self, inventory: &mut [Good]) {
        let mut sold_tokens = self.sold_tokens.borrow_mut();
        let mut sell_tokens = self.sell_tokens.borrow_mut();

        // loop over all sell tokens and sell the good
        for (token, offer) in sell_tokens.iter_mut() {
            // We can be sure that this market exist
            let market = self.find_market_for_name(&offer.market_name).unwrap();
            let mut market = market.as_ref().borrow_mut();

            let good = self
                .get_mut_good_for_kind(&offer.good_kind, inventory)
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
                    offer.market_name
                );
                // add (remove) from history
                self.add_to_buy_history(
                    &Good::new(offer.good_kind, new_quantity * (-1.0)), // TODO Check this
                    cash.get_qty() * (-1.0),
                );
                // Now increase our eur quantity
                let eur = self
                    .get_mut_good_for_kind(&GoodKind::EUR, inventory)
                    .unwrap();
                let _ = eur.merge(cash);
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
    /// This method clears all token from `sell_tokens` that are contained in `sold_tokens`.
    fn clear_sold_tokens(&self) {
        let mut sell_tokens = self.sell_tokens.borrow_mut();
        let sold_tokens = self.sold_tokens.borrow();

        for sold_token in sold_tokens.iter() {
            if let Some(index) = sell_tokens
                .iter()
                .position(|(token, _)| *token == *sold_token)
            {
                // token exist
                sell_tokens.remove(index);
            }
        }
    }
}

/// Helper methods
impl AverageSellerStrategy {
    /// Builds a default buy history that contains all tradable goods.
    fn init_default_buy_history() -> BuyHistory {
        // don't care about EUR
        let kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];

        let mut history: BuyHistory = HashMap::new();
        for kind in kinds {
            history.insert(kind, Vec::new());
        }

        history
    }

    /// Returns a mutable reference to the wanted good
    fn get_mut_good_for_kind<'a>(
        &'a self,
        kind: &GoodKind,
        inventory: &'a mut [Good],
    ) -> Option<&mut Good> {
        inventory.iter_mut().find(|g| g.get_kind() == *kind)
    }

    /// Returns a reference to the wanted good if available
    fn get_good_for_kind<'a>(&'a self, kind: &GoodKind, inventory: &'a [Good]) -> Option<&Good> {
        inventory.iter().find(|g| g.get_kind() == *kind)
    }

    /// Returns an optional ref the market, if a market for the given name as found.
    fn find_market_for_name(&self, name: &String) -> Option<MarketRef> {
        self.markets
            .iter()
            .find(|m| &m.as_ref().borrow().get_name().to_string() == name)
            .map(Rc::clone)
    }

    /// Returns the average price per piece this strategy has paid for a single piece of
    /// the given good kind.
    fn get_avg_buy_price_per_piece(&self, kind: &GoodKind) -> f32 {
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
impl Strategy for AverageSellerStrategy {
    fn new(markets: Vec<MarketRef>, trader_name: &str) -> Self {
        Self {
            trader_name: trader_name.to_string(),
            markets,
            buy_tokens: RefCell::new(Vec::new()),
            bought_tokens: RefCell::new(Vec::new()),
            buy_history: RefCell::new(AverageSellerStrategy::init_default_buy_history()),
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
        // Try to sell everything we have for the best price possible
        let offers = self.find_offers_for_markets(goods, |market, good| {
            let market = market.as_ref().borrow();
            // Just return the offer for the max quantity
            if let Ok(price) = market.get_sell_price(good.get_kind(), good.get_qty()) {
                let market_name = market.get_name().to_string();
                Some(Payment::new(
                    price,
                    good.get_qty(),
                    good.get_kind(),
                    market_name,
                ))
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
        self.lock_bids(goods); // 1. Lock buy the cheapest good we can find
        self.buy_locked_goods(goods); // 2. Buy all locked goods
        self.clear_bought_tokens(); // 3. Clear buy tokens
        self.lock_goods_for_sell(goods); // 4. Lock sell all goods for a higher price
        self.sell_locked_goods(goods); // 5. Sell our goods
        self.clear_sold_tokens(); // 6. Clear sell tokens
    }
}

#[cfg(test)]
mod tests {
    use crate::strategies::average_seller_strategy::{AverageSellerStrategy, Payment};
    use crate::strategies::strategy::Strategy;
    use crate::MarketRef;
    use smse::Smse;
    use std::cell::RefCell;
    use std::rc::Rc;
    use unitn_market_2022::good::good::Good;
    use unitn_market_2022::good::good_kind::GoodKind;
    use unitn_market_2022::market::Market;
    use SGX::market::sgx::SGX;
    use TASE::TASE;
    use ZSE::market::ZSE;

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

    fn init_random_markets() -> (MarketRef, MarketRef, MarketRef, MarketRef) {
        let sgx = SGX::new_random();
        let smse = Smse::new_random();
        let tase = TASE::new_random();
        let zse = ZSE::new_random();
        (sgx, smse, tase, zse)
    }

    fn init_inventory(
        eur_quantity: f32,
        usd_quantity: f32,
        yen_quantity: f32,
        yuan_quantity: f32,
    ) -> Vec<Good> {
        vec![
            Good::new(GoodKind::EUR, eur_quantity),
            Good::new(GoodKind::USD, usd_quantity),
            Good::new(GoodKind::YEN, yen_quantity),
            Good::new(GoodKind::YUAN, yuan_quantity),
        ]
    }

    #[test]
    fn test_find_adequate_offer() {
        let trader_name = "TRADER_NAME";
        let (sgx, _, _, _) = init_random_markets();
        let markets = vec![Rc::clone(&sgx)];
        let strategy = AverageSellerStrategy::new(markets, trader_name);

        // test for eur
        let good = Good::new(GoodKind::EUR, 100.0);
        let offer = strategy.find_adequate_offer(Rc::clone(&sgx), &good);
        assert!(offer.is_none(), "There shouldn't be any offer for EUR");

        // test with 0.0 quantity
        let good = Good::new(GoodKind::USD, 0.0);
        let offer = strategy.find_adequate_offer(Rc::clone(&sgx), &good);
        assert!(
            offer.is_none(),
            "There shouldn't be any offer for for an empty quantity"
        );

        // test with low USD
        let good = Good::new(GoodKind::USD, 1.0);
        let sgx_ref = sgx.as_ref().borrow();
        let sell_price = sgx_ref
            .get_sell_price(good.get_kind(), good.get_qty())
            .unwrap();
        let offer = strategy.find_adequate_offer(Rc::clone(&sgx), &good);
        assert!(offer.is_some(), "There should be an offer for 1.0 USD");
        let offer = offer.unwrap();
        let market_name = sgx_ref.get_name();
        assert_eq!(
            market_name, offer.market_name,
            "Market name of offer must be {}",
            market_name
        );
        assert_eq!(
            good.get_kind(),
            offer.good_kind,
            "Good kind of offer must be {}",
            good.get_kind()
        );
        assert_eq!(
            good.get_qty(),
            offer.quantity,
            "Offer quantity must be equal to {}",
            good.get_qty()
        );
        assert_eq!(
            sell_price, offer.price,
            "Price of offer must be {}",
            sell_price
        );
    }

    #[test]
    fn test_find_adequate_bid() {
        let trader_name = "TRADER_NAME";
        let (_, smse, tase, _) = init_markets(0.0, 100_000.0, 0.0, 0.0);
        let markets = vec![Rc::clone(&smse), Rc::clone(&tase)];
        let strategy = AverageSellerStrategy::new(markets, trader_name);

        // test for eur
        let offer = strategy.find_adequate_bid(Rc::clone(&smse), 100.0, &GoodKind::EUR);
        assert!(offer.is_none(), "There shouldn't be any offer for EUR");

        // test with 0.0 max_eur
        let offer = strategy.find_adequate_bid(Rc::clone(&smse), 0.0, &GoodKind::USD);
        assert!(
            offer.is_none(),
            "There shouldn't be any offer for for an empty quantity"
        );

        // test with max USD
        let smse_ref = tase.as_ref().borrow();
        let max_usd_qty = 100_000.0;
        let max_buy_price = smse_ref.get_buy_price(GoodKind::USD, max_usd_qty).unwrap();
        let bid = strategy.find_adequate_bid(Rc::clone(&tase), max_buy_price, &GoodKind::USD);
        assert!(
            bid.is_some(),
            "There should be a bid for {} EUR",
            max_buy_price
        );
        let bid = bid.unwrap();
        let market_name = smse_ref.get_name();
        assert_eq!(
            market_name, bid.market_name,
            "Market name of bid must be {}",
            market_name
        );
        assert_eq!(
            GoodKind::USD,
            bid.good_kind,
            "Good kind of offer must be USD"
        );
        assert!(
            bid.quantity <= max_usd_qty,
            "Bid quantity must be smaller of equal to max. quantity of {} USD (is: {} USD)",
            max_usd_qty,
            bid.quantity
        );
        assert!(
            bid.price <= max_buy_price,
            "Price of bid must be smaller or equal to max. price of {} EUR (is: {} EUR)",
            max_buy_price,
            bid.price
        );
    }

    #[test]
    fn test_filter_best_offers() {
        let trader_name = "TRADER_NAME";
        let strategy = AverageSellerStrategy::new(Vec::new(), trader_name);

        // test with empty offers
        let best_offers = strategy.filter_best_offers(&[]);
        assert_eq!(
            0,
            best_offers.len(),
            "There shouldn't be any best offers for no offers"
        );

        // test with only USD offers
        let usd_offer_a = Payment::new(100.0, 2000.0, GoodKind::USD, "market_a".to_string());
        let usd_offer_b = Payment::new(100_000.0, 80_000.0, GoodKind::USD, "market_b".to_string());
        let usd_offer_c = Payment::new(1.0, 5.0, GoodKind::USD, "market_c".to_string());
        let usd_offer_d = Payment::new(1_000_000.0, 100.0, GoodKind::USD, "market_d".to_string());
        let offers = vec![usd_offer_a, usd_offer_b, usd_offer_c, usd_offer_d.clone()];
        let best_offers = strategy.filter_best_offers(&offers);
        assert_eq!(1, best_offers.len(), "There should be only be one offer");
        let best_offer = best_offers.first().unwrap();
        assert_eq!(
            usd_offer_d.good_kind, best_offer.good_kind,
            "Best offer kind must be {}",
            usd_offer_d.good_kind
        );
        assert_eq!(
            usd_offer_d.price, best_offer.price,
            "Best offer price must be {}",
            usd_offer_d.price
        );
        assert_eq!(
            usd_offer_d.quantity, best_offer.quantity,
            "Best offer quantity must be {}",
            usd_offer_d.quantity
        );
        assert_eq!(
            usd_offer_d.market_name, best_offer.market_name,
            "Best offer market must be {}",
            usd_offer_d.market_name
        );

        // test with many offers
        let offer_a = Payment::new(100.0, 2000.0, GoodKind::YEN, "market_a".to_string());
        let offer_b = Payment::new(100_000.0, 80_000.0, GoodKind::YUAN, "market_b".to_string());
        let offer_c = Payment::new(1.0, 5.0, GoodKind::USD, "market_c".to_string());
        let offer_d = Payment::new(1_500_000.0, 100.0, GoodKind::YEN, "market_d".to_string());
        let offer_e = Payment::new(2_000_000.0, 2000.0, GoodKind::YUAN, "market_a".to_string());
        let offer_f = Payment::new(3_450_000.0, 80_000.0, GoodKind::USD, "market_b".to_string());
        let offers = vec![
            offer_a,
            offer_b,
            offer_c,
            offer_d.clone(),
            offer_e.clone(),
            offer_f.clone(),
        ];
        let best_offers = strategy.filter_best_offers(&offers);
        assert_eq!(
            3,
            best_offers.len(),
            "There should be a best offer for every kind"
        );

        let usd_best_offer = best_offers
            .iter()
            .find(|o| o.good_kind == GoodKind::USD)
            .unwrap();
        assert_eq!(
            offer_f.good_kind, usd_best_offer.good_kind,
            "USD Best offer kind must be {}",
            offer_f.good_kind
        );
        assert_eq!(
            offer_f.price, usd_best_offer.price,
            "USD Best offer price must be {} EUR",
            offer_f.price
        );
        assert_eq!(
            offer_f.quantity, usd_best_offer.quantity,
            "USD Best offer quantity must be {} USD",
            offer_f.quantity
        );
        assert_eq!(
            offer_f.market_name, usd_best_offer.market_name,
            "USD Best offer market must be {}",
            offer_f.market_name
        );

        let yen_best_offer = best_offers
            .iter()
            .find(|o| o.good_kind == GoodKind::YEN)
            .unwrap();
        assert_eq!(
            offer_d.good_kind, yen_best_offer.good_kind,
            "YEN Best offer kind must be {}",
            offer_d.good_kind
        );
        assert_eq!(
            offer_d.price, yen_best_offer.price,
            "YEN Best offer price must be {} EUR",
            offer_d.price
        );
        assert_eq!(
            offer_d.quantity, yen_best_offer.quantity,
            "YEN Best offer quantity must be {} YEN",
            offer_d.quantity
        );
        assert_eq!(
            offer_d.market_name, yen_best_offer.market_name,
            "YEN Best offer market must be {}",
            offer_d.market_name
        );

        let yuan_best_offer = best_offers
            .iter()
            .find(|o| o.good_kind == GoodKind::YUAN)
            .unwrap();
        assert_eq!(
            offer_e.good_kind, yuan_best_offer.good_kind,
            "YUAN Best offer kind must be {}",
            offer_e.good_kind
        );
        assert_eq!(
            offer_e.price, yuan_best_offer.price,
            "YUAN Best offer price must be {} EUR",
            offer_e.price
        );
        assert_eq!(
            offer_e.quantity, yuan_best_offer.quantity,
            "YUAN Best offer quantity must be {} YUAN",
            offer_e.quantity
        );
        assert_eq!(
            offer_e.market_name, yuan_best_offer.market_name,
            "YUAN Best offer market must be {}",
            offer_e.market_name
        );
    }

    #[test]
    fn test_find_adequate_bids() {
        let trader_name = "TRADER_NAME";
        let (sgx, _, _, _) = init_markets(0.0, 100_000.0, 0.0, 0.0);
        let markets = vec![Rc::clone(&sgx)];
        let strategy = AverageSellerStrategy::new(markets, trader_name);

        // test with eur
        let bids = strategy.find_adequate_bids(&GoodKind::EUR, 100_000.0, |_, _, _| None);
        assert_eq!(0, bids.len(), "There shouldn't be any bids for EUR");

        // test with 0.0 max eur
        let bids = strategy.find_adequate_bids(&GoodKind::USD, 0.0, |_, _, _| None);
        assert_eq!(
            0,
            bids.len(),
            "There shouldn't be any bids for 0.0 EUR max. EUR price"
        );

        // test with a list of multiple bids
        let max_eur = 1_000_000.0;
        let best_bid = strategy
            .find_adequate_bid(Rc::clone(&sgx), max_eur, &GoodKind::USD)
            .unwrap();
        let best_bids = strategy.find_adequate_bids(&GoodKind::USD, max_eur, |m, e, k| {
            strategy.find_adequate_bid(m, e, k)
        });
        assert!(!best_bids.is_empty(), "There must be at least one best bid");
        let best_usd_bid = best_bids.iter().find(|p| p.good_kind == GoodKind::USD);
        assert!(best_usd_bid.is_some(), "The must be a best bid for USD");
        let best_usd_bid = best_usd_bid.unwrap();
        assert_eq!(
            best_bid.good_kind, best_usd_bid.good_kind,
            "Best bid kind must be {}",
            best_bid.good_kind
        );
        assert_eq!(
            best_bid.price, best_usd_bid.price,
            "Best bid price for USD must be {} EUR",
            best_bid.price
        );
        assert_eq!(
            best_bid.quantity, best_usd_bid.quantity,
            "Best bid quantity for USD must be {} USD",
            best_bid.quantity
        );
        assert_eq!(
            best_bid.market_name, best_usd_bid.market_name,
            "Best offer market for USD must be {}",
            best_bid.market_name
        );
    }

    #[test]
    fn test_find_good_to_lock_buy() {
        let trader_name = "TRADER_NAME";
        let strategy = AverageSellerStrategy::new(vec![], trader_name);
        let allowed_kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];

        // Test with all goods at 0.0 quantity
        let inventory = init_inventory(0.0, 0.0, 0.0, 0.0);
        let kind = strategy.find_good_to_lock_buy(&inventory);
        assert_ne!(GoodKind::EUR, kind, "Kind to buy can never be EUR");
        assert!(
            allowed_kinds.contains(&kind),
            "Kind can be USD, YEN, or YUAN"
        );

        // Test with only EUR at 0.0 quantity
        let inventory = init_inventory(0.0, 100.0, 100.0, 100.0);
        let kind = strategy.find_good_to_lock_buy(&inventory);
        assert_ne!(GoodKind::EUR, kind, "Kind to buy can never be EUR");
        assert!(
            allowed_kinds.contains(&kind),
            "Kind can be USD, YEN, or YUAN"
        );

        // Test with only YUAN at 0.0 quantity
        let inventory = init_inventory(100.0, 0.0, 100.0, 0.0);
        let kind = strategy.find_good_to_lock_buy(&inventory);
        assert_ne!(GoodKind::EUR, kind, "Kind to buy can never be EUR");
        assert!(
            vec![GoodKind::USD, GoodKind::YUAN].contains(&kind),
            "Kind must be USD or YUAN"
        );

        // Test with USD and YUAN at 0.0 quantity
        let inventory = init_inventory(100.0, 100.0, 100.0, 0.0);
        let kind = strategy.find_good_to_lock_buy(&inventory);
        assert_ne!(GoodKind::EUR, kind, "Kind to buy can never be EUR");
        assert_eq!(GoodKind::YUAN, kind, "Kind must be YUAN");
    }

    #[test]
    fn test_allowed_to_buy() {
        let trader_name = "TRADER_NAME";
        let mut strategy = AverageSellerStrategy::new(vec![], trader_name);

        // at first we are allowed to buy
        assert!(
            strategy.allowed_to_buy(),
            "At first, the strategy must be allowed to buy"
        );

        strategy.buy_count = RefCell::new(6);
        assert!(
            !strategy.allowed_to_buy(),
            "After {} buy operations and {} sell operations it should not be allowed to buy",
            6,
            0
        );

        strategy.sell_count = RefCell::new(6);
        assert!(
            strategy.allowed_to_buy(),
            "After {} buy operations and {} sell operations it should not be allowed to buy",
            6,
            6
        );

        strategy.buy_count = RefCell::new(60);
        strategy.sell_count = RefCell::new(10);
        assert!(
            !strategy.allowed_to_buy(),
            "After {} buy operations and {} sell operations it should not be allowed to buy",
            60,
            10
        );
    }

    #[test]
    fn test_get_good_for_kind() {
        let trader_name = "TRADER_NAME";
        let strategy = AverageSellerStrategy::new(vec![], trader_name);
        let inventory = init_inventory(0.0, 0.0, 0.0, 0.0);

        let kinds = vec![GoodKind::EUR, GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
        for kind in kinds.iter() {
            let good = strategy.get_good_for_kind(kind, &inventory);
            assert!(good.is_some(), "There must be a good for kind {}", kind);
            let good = good.unwrap();
            assert_eq!(*kind, good.get_kind(), "Good must be of kind {}", kind);
            assert_eq!(0.0, good.get_qty(), "Quantity of good is 0.0");
        }
    }

    #[test]
    fn test_add_to_buy_history() {
        let trader_name = "TRADER_NAME";
        let strategy = AverageSellerStrategy::new(vec![], trader_name);

        // initially everything must be empty
        for (kind, hist) in strategy.buy_history.borrow().iter() {
            assert!(hist.is_empty(), "History for {kind} should be empty");
        }

        // add goods to buy history
        let kinds = vec![GoodKind::USD, GoodKind::YEN, GoodKind::YUAN];
        for kind in kinds {
            strategy.add_to_buy_history(&Good::new(kind, 10.0), 100.0);
        }
        for (kind, hist) in strategy.buy_history.borrow().iter() {
            assert_eq!(1, hist.len(), "History length for {kind} should be 1");
        }
    }

    #[test]
    fn test_find_market_for_name() {
        let trader_name = "TRADER_NAME";
        let (sgx, smse, tase, zse) = init_random_markets();
        let markets = vec![
            Rc::clone(&sgx),
            Rc::clone(&smse),
            Rc::clone(&tase),
            Rc::clone(&zse),
        ];
        let strategy = AverageSellerStrategy::new(markets, trader_name);

        // Test SGX
        let sgx_name = sgx.as_ref().borrow().get_name();
        let sgx_2 = strategy.find_market_for_name(&sgx_name.to_string());
        assert!(
            sgx_2.is_some(),
            "There must be a market for name '{}'",
            sgx_name
        );
        let sgx_2_name = sgx_2.unwrap().as_ref().borrow().get_name();
        assert_eq!(
            sgx_name, sgx_2_name,
            "Found name '{}' must be equal to '{}'",
            sgx_2_name, sgx_name
        );

        // Test SMSE
        let smse_name = smse.as_ref().borrow().get_name();
        let smse_2 = strategy.find_market_for_name(&smse_name.to_string());
        assert!(
            smse_2.is_some(),
            "There must be a market for name '{}'",
            smse_name
        );
        let smse_2_name = smse_2.unwrap().as_ref().borrow().get_name();
        assert_eq!(
            smse_name, smse_2_name,
            "Found name '{}' must be equal to '{}'",
            smse_2_name, smse_name
        );

        // Test TASE
        let tase_name = tase.as_ref().borrow().get_name();
        let tase_2 = strategy.find_market_for_name(&tase_name.to_string());
        assert!(
            tase_2.is_some(),
            "There must be a market for name '{}'",
            tase_name
        );
        let tase_2_name = tase_2.unwrap().as_ref().borrow().get_name();
        assert_eq!(
            tase_name, tase_name,
            "Found name '{}' must be equal to '{}'",
            tase_2_name, tase_name
        );

        // Test ZSE
        let zse_name = zse.as_ref().borrow().get_name();
        let zse_2 = strategy.find_market_for_name(&zse_name.to_string());
        assert!(
            zse_2.is_some(),
            "There must be a market for name '{}'",
            zse_name
        );
        let zse_2_name = zse_2.unwrap().as_ref().borrow().get_name();
        assert_eq!(
            zse_name, zse_2_name,
            "Found name '{}' must be equal to '{}'",
            zse_2_name, zse_name
        );
    }

    #[test]
    fn test_clear_tokens() {
        let trader_name = "TRADER_NAME";
        let mut strategy = AverageSellerStrategy::new(vec![], trader_name);

        // Initially no tokens should be available
        assert!(
            strategy.sell_tokens.borrow().is_empty(),
            "Initially it must be empty"
        );
        assert!(
            strategy.sold_tokens.borrow().is_empty(),
            "Initially it must be empty"
        );
        assert!(
            strategy.buy_tokens.borrow().is_empty(),
            "Initially it must be empty"
        );
        assert!(
            strategy.bought_tokens.borrow().is_empty(),
            "Initially it must be empty"
        );

        let tokens = vec![
            (
                "TOKEN_A".to_string(),
                Payment::new(10.0, 100.0, GoodKind::USD, "MARKET_A".to_string()),
            ),
            (
                "TOKEN_B".to_string(),
                Payment::new(10.0, 100.0, GoodKind::USD, "MARKET_B".to_string()),
            ),
        ];

        // add the some tokens to both
        for (token, payment) in tokens.iter() {
            strategy
                .sell_tokens
                .borrow_mut()
                .push((token.clone(), payment.clone()));
            strategy.sold_tokens.borrow_mut().push(token.clone());
            strategy
                .buy_tokens
                .borrow_mut()
                .push((token.clone(), payment.clone()));
            strategy.bought_tokens.borrow_mut().push(token.clone());
        }

        // Initially no tokens should be available
        assert_eq!(
            2,
            strategy.sell_tokens.borrow().len(),
            "Length must be 2 now"
        );
        assert_eq!(
            2,
            strategy.sold_tokens.borrow().len(),
            "Length must be 2 now"
        );
        assert_eq!(
            2,
            strategy.buy_tokens.borrow().len(),
            "Length must be 2 now"
        );
        assert_eq!(
            2,
            strategy.bought_tokens.borrow().len(),
            "Length must be 2 now"
        );

        strategy.clear_sold_tokens();
        strategy.clear_bought_tokens();

        assert!(
            strategy.sell_tokens.borrow().is_empty(),
            "After clear, history must be empty"
        );
        assert!(
            strategy.buy_tokens.borrow().is_empty(),
            "After clear, history must be empty"
        );

        // clear completely
        strategy.sold_tokens = RefCell::new(vec![]);
        strategy.bought_tokens = RefCell::new(vec![]);

        // add only one token to sold/bought tokens
        for (index, (token, payment)) in tokens.iter().enumerate() {
            if index % 2 == 0 {
                strategy.sold_tokens.borrow_mut().push(token.clone());
                strategy.bought_tokens.borrow_mut().push(token.clone());
            }
            strategy
                .sell_tokens
                .borrow_mut()
                .push((token.clone(), payment.clone()));
            strategy
                .buy_tokens
                .borrow_mut()
                .push((token.clone(), payment.clone()));
        }

        assert_eq!(
            2,
            strategy.sell_tokens.borrow().len(),
            "Length must be 2 now"
        );
        assert_eq!(
            1,
            strategy.sold_tokens.borrow().len(),
            "Length must be 1 now"
        );
        assert_eq!(
            2,
            strategy.buy_tokens.borrow().len(),
            "Length must be 2 now"
        );
        assert_eq!(
            1,
            strategy.bought_tokens.borrow().len(),
            "Length must be 1 now"
        );

        strategy.clear_sold_tokens();
        strategy.clear_bought_tokens();

        assert_eq!(
            1,
            strategy.sell_tokens.borrow().len(),
            "After clear, length must be 1"
        );
        assert_eq!(
            1,
            strategy.buy_tokens.borrow().len(),
            "After clear, length must be 1"
        );
    }
}
