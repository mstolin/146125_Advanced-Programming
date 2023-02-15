use std::cell::RefCell;
use std::rc::Rc;

use unitn_market_2022::event::event::{Event, EventKind};
use unitn_market_2022::event::notifiable::Notifiable;
use unitn_market_2022::good::consts::*;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::good_label::GoodLabel;
use unitn_market_2022::market::{
    BuyError, LockBuyError, LockSellError, Market, MarketGetterError, SellError,
};

use crate::goods::good_storage::GoodStorage;
use crate::market::consts::NAME;
use crate::market::log::{
    log_for_buy, log_for_buy_err, log_for_lock_buy, log_for_lock_buy_err, log_for_lock_sell,
    log_for_lock_sell_err, log_for_market_init, log_for_sell, log_for_sell_err,
};

pub struct SGX {
    good_storage: GoodStorage,
    subscribers: Vec<Box<dyn Notifiable>>,
}

impl SGX {
    /// Notifies all subscribers of the given event
    fn notify(&mut self, event: Event) {
        self.subscribers
            .iter_mut()
            .for_each(|s| s.as_mut().on_event(event.clone()));
    }

    /// Merges the default good with the given good
    fn merge_default_good(&mut self, other: Good) {
        if let Some((our_eur, _)) = self.good_storage.get_mut_good_for_kind(&DEFAULT_GOOD_KIND) {
            let _ = our_eur.merge(other);
        }
    }

    /// Splits the default good with the given quantity
    fn split_default_good(&mut self, eur_quantity: f32) {
        if let Some((good, _)) = self.good_storage.get_mut_good_for_kind(&DEFAULT_GOOD_KIND) {
            let _ = good.split(eur_quantity);
        }
    }

    /// Checks if another Good can be locked for buy
    fn can_add_buy_lock(&self) -> bool {
        self.good_storage.get_buy_locks_len() < self.good_storage.len() - 2
    }

    /// Checks if another Good can be locked for sell
    fn can_add_sell_lock(&self) -> bool {
        self.good_storage.get_sell_locks_len() < self.good_storage.len() - 2
    }
}

impl Notifiable for SGX {
    fn add_subscriber(&mut self, subscriber: Box<dyn Notifiable>) {
        self.subscribers.push(subscriber);
    }

    fn on_event(&mut self, event: Event) {
        match event.kind {
            EventKind::Bought | EventKind::LockedBuy => {
                let our_price = match self.get_buy_price(event.good_kind, event.quantity) {
                    Ok(our_price) => our_price,
                    _ => {
                        return; // not able get our price
                    }
                };

                let (_, bought_meta) =
                    match self.good_storage.get_mut_good_for_kind(&event.good_kind) {
                        Some((good, meta)) => (good, meta),
                        _ => return, // no good found
                    };
                bought_meta.fluctuate_sell_price_with_factor(1.05);
                // only decrease buy if our price is already higher
                if our_price > event.price {
                    let factor = event.price / our_price;
                    bought_meta.fluctuate_buy_price_with_factor(factor);
                }
            }
            EventKind::Sold | EventKind::LockedSell => {
                // a good was sold => demand to sell increases => lower our prices
                let our_price = match self.get_sell_price(event.good_kind, event.quantity) {
                    Ok(our_price) => our_price,
                    _ => {
                        return; // not able get our price
                    }
                };

                let (_, bought_meta) =
                    match self.good_storage.get_mut_good_for_kind(&event.good_kind) {
                        Some((good, meta)) => (good, meta),
                        _ => return, // no good found
                    };
                bought_meta.fluctuate_buy_price_with_factor(1.05);
                // only decrease buy if our price is already higher
                if our_price > event.price {
                    let factor = event.price / our_price;
                    bought_meta.fluctuate_sell_price_with_factor(factor);
                }
            }
            EventKind::Wait => {
                // A day has passed, we may decrease our prices
                let iter = self.good_storage.iter_mut();
                for (good, meta) in iter {
                    // fluctuate prices each day
                    if good.get_kind() != DEFAULT_GOOD_KIND {
                        meta.fluctuate_buy_price_with_factor(0.9); // buy price always cheaper
                        meta.fluctuate_sell_price_with_factor(0.95);
                    }
                    // unlock for sell if needed
                    if let Some(lock) = meta.get_mut_sell_lock() {
                        if lock.age_in_days < 15 {
                            lock.increase_age_by_one();
                        } else {
                            meta.unlock_for_sell();
                        }
                    }
                    // unlock for buy if needed
                    if let Some(lock) = meta.get_mut_buy_lock() {
                        if lock.age_in_days < 15 {
                            lock.increase_age_by_one();
                        } else {
                            meta.unlock_for_buy();
                        }
                    }
                }
            }
        }
    }
}

impl Market for SGX {
    fn new_random() -> Rc<RefCell<dyn Market>> {
        let good_storage = GoodStorage::new_random(STARTING_CAPITAL);
        let (eur, _) = good_storage.get_good_for_kind(&GoodKind::EUR).unwrap();
        let (usd, _) = good_storage.get_good_for_kind(&GoodKind::USD).unwrap();
        let (yen, _) = good_storage.get_good_for_kind(&GoodKind::YEN).unwrap();
        let (yuan, _) = good_storage.get_good_for_kind(&GoodKind::YUAN).unwrap();
        log_for_market_init(eur.get_qty(), yen.get_qty(), usd.get_qty(), yuan.get_qty());
        let rand_market = SGX {
            good_storage,
            subscribers: Vec::new(),
        };
        Rc::new(RefCell::new(rand_market))
    }

    fn new_with_quantities(eur: f32, yen: f32, usd: f32, yuan: f32) -> Rc<RefCell<dyn Market>> {
        log_for_market_init(eur, yen, usd, yuan);
        let market = SGX {
            good_storage: GoodStorage::with_quantities(eur, yen, usd, yuan),
            subscribers: Vec::new(),
        };
        Rc::new(RefCell::new(market))
    }

    fn new_file(_path: &str) -> Rc<RefCell<dyn Market>> {
        todo!()
    }

    fn get_name(&self) -> &'static str {
        NAME
    }

    fn get_budget(&self) -> f32 {
        self.good_storage.get_default_good().0.get_qty()
    }

    fn get_buy_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        if quantity <= 0.0 {
            return Err(MarketGetterError::NonPositiveQuantityAsked);
        }

        let (good, meta) = self
            .good_storage
            .get_good_for_kind(&kind)
            .expect("Not able to get good for buy price");
        let available_good_quantity = good.get_qty();

        if available_good_quantity < quantity {
            return Err(MarketGetterError::InsufficientGoodQuantityAvailable {
                requested_good_kind: kind,
                available_good_quantity,
                requested_good_quantity: quantity,
            });
        }

        // Get buy price for quantity
        let exchange_price = quantity * meta.base_buy_price;
        // Calculate a factor based on demand
        let new_quantity = available_good_quantity - quantity;
        let demand_factor = available_good_quantity / new_quantity;
        // Calculate a margin (buy price must always be cheaper than sell price)
        let margin = exchange_price * 0.05;

        let price = (exchange_price + margin) * demand_factor;
        Ok(price)
    }

    fn get_sell_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError> {
        if quantity <= 0.0 {
            return Err(MarketGetterError::NonPositiveQuantityAsked);
        }

        let (good, meta) = self
            .good_storage
            .get_good_for_kind(&kind)
            .expect("Not able to get good for sell price");
        let available_good_quantity = good.get_qty();

        // Get sell price for quantity
        let exchange_price = quantity * meta.base_sell_price;
        // Calculate a factor based on demand
        let new_quantity = available_good_quantity + quantity;
        let demand_factor = available_good_quantity / new_quantity;
        // Calculate a margin (sell price must always be higher than buy price)
        let margin = exchange_price * 0.15;

        let price = (exchange_price + margin) * demand_factor;
        Ok(price)
    }

    fn get_goods(&self) -> Vec<GoodLabel> {
        self.good_storage.get_good_labels()
    }

    fn lock_buy(
        &mut self,
        kind_to_buy: GoodKind,
        quantity_to_buy: f32,
        bid: f32,
        trader_name: String,
    ) -> Result<String, LockBuyError> {
        // First check if good is available
        let (good, meta) = match self.good_storage.get_good_for_kind(&kind_to_buy) {
            Some((good, meta)) => (good, meta),
            _ => {
                panic!(
                    "Good {} can't be locked for buy, because it was not found",
                    kind_to_buy
                )
            }
        };

        // Is the Good already locked?
        if let Some(lock) = meta.get_buy_lock() {
            log_for_lock_buy_err(trader_name, kind_to_buy, quantity_to_buy, bid);
            return Err(LockBuyError::GoodAlreadyLocked {
                token: lock.transaction_token.clone(),
            });
        }

        // Is the number of max. locks achieved
        if !self.can_add_buy_lock() {
            log_for_lock_buy_err(trader_name, kind_to_buy, quantity_to_buy, bid);
            return Err(LockBuyError::MaxAllowedLocksReached);
        }

        // Not locked, but is the quantity enough?
        if quantity_to_buy <= 0.0 {
            log_for_lock_buy_err(trader_name, kind_to_buy, quantity_to_buy, bid);
            return Err(LockBuyError::NonPositiveQuantityToBuy {
                negative_quantity_to_buy: quantity_to_buy,
            });
        } else if quantity_to_buy > good.get_qty() {
            log_for_lock_buy_err(trader_name, kind_to_buy, quantity_to_buy, bid);
            return Err(LockBuyError::InsufficientGoodQuantityAvailable {
                requested_good_kind: kind_to_buy,
                requested_good_quantity: quantity_to_buy,
                available_good_quantity: good.get_qty(),
            });
        }

        let lowest_acceptable_bid = match self.get_buy_price(kind_to_buy, quantity_to_buy) {
            Ok(price) => price,
            Err(_) => {
                log_for_lock_buy_err(trader_name, kind_to_buy, quantity_to_buy, bid);
                return Err(LockBuyError::InsufficientGoodQuantityAvailable {
                    requested_good_kind: kind_to_buy,
                    requested_good_quantity: quantity_to_buy,
                    available_good_quantity: good.get_qty(),
                });
            }
        };

        if bid <= 0.0 {
            log_for_lock_buy_err(trader_name, kind_to_buy, quantity_to_buy, bid);
            return Err(LockBuyError::NonPositiveBid { negative_bid: bid });
        } else if bid < lowest_acceptable_bid {
            log_for_lock_buy_err(trader_name, kind_to_buy, quantity_to_buy, bid);
            return Err(LockBuyError::BidTooLow {
                requested_good_kind: kind_to_buy,
                requested_good_quantity: quantity_to_buy,
                low_bid: bid,
                lowest_acceptable_bid,
            });
        }

        // Now we can lock, we are able to unwrap here, because we checked earlier
        let (good, meta) = self
            .good_storage
            .get_mut_good_for_kind(&kind_to_buy)
            .expect("Not able to get mutable ref for good");

        // Lock the Good (update its metadata)
        let token = meta.lock_for_buy(quantity_to_buy, kind_to_buy, bid, trader_name.clone());

        // fluctuation
        let new_quantity = good.get_qty() - quantity_to_buy;
        let factor = good.get_qty() / new_quantity;
        meta.fluctuate_buy_price_with_factor(factor);

        // notify
        let event = Event {
            kind: EventKind::LockedBuy,
            good_kind: kind_to_buy,
            quantity: quantity_to_buy,
            price: bid,
        };
        self.notify(event);

        log_for_lock_buy(
            trader_name,
            kind_to_buy,
            quantity_to_buy,
            bid,
            token.clone(),
        );
        Ok(token)
    }

    /// Call when a trader **buys from this market**
    fn buy(&mut self, token: String, cash: &mut Good) -> Result<Good, BuyError> {
        // get the good as mut that was locked (in lock_buy) with the given token
        let (locked_good, locked_meta) = match self.good_storage.get_mut_good_for_buy_token(&token)
        {
            Some((good, meta)) => (good, meta),
            _ => {
                return if self.good_storage.has_good_expired_buy_token(&token) {
                    log_for_buy_err(token.clone());
                    Err(BuyError::ExpiredToken {
                        expired_token: token,
                    })
                } else {
                    // this token is invalid
                    log_for_buy_err(token.clone());
                    Err(BuyError::UnrecognizedToken {
                        unrecognized_token: token,
                    })
                };
            }
        };

        // Get the lock
        let lock = locked_meta
            .get_buy_lock()
            .expect("Not able to get lock from metadata")
            .clone();

        // check if cash is of default kind (we only sell for EUR)
        if cash.get_kind() != DEFAULT_GOOD_KIND {
            locked_meta.unlock_for_buy();
            log_for_buy_err(token.clone());
            return Err(BuyError::GoodKindNotDefault {
                non_default_good_kind: cash.get_kind(),
            });
        }

        // check if cash quantity is at least equal the agreed price
        if cash.get_qty() < lock.eur_quantity {
            locked_meta.unlock_for_buy();
            log_for_buy_err(token.clone());
            return Err(BuyError::InsufficientGoodQuantity {
                contained_quantity: cash.get_qty(),
                pre_agreed_quantity: lock.eur_quantity,
            });
        }

        // save old old quantity
        let old_quantity = locked_good.get_qty();
        // Split the good, this will also mutate (decrease the amount that this market sells) from the locked good
        let splitted_good = locked_good.split(lock.locked_original_qty);
        if let Ok(splitted_good) = splitted_good {
            // fluctuation
            let new_quantity = old_quantity - lock.locked_original_qty;
            let factor = old_quantity / new_quantity;
            locked_meta.fluctuate_buy_price_with_factor(factor);

            // unlock good
            locked_meta.unlock_for_buy();

            // update default good (increase our EUR with cash)
            self.merge_default_good(cash.clone());
            // split buy price from buyers cash
            let _ = cash.split(lock.eur_quantity);

            // notify
            let event = Event {
                kind: EventKind::Bought,
                good_kind: lock.kind,
                quantity: lock.locked_original_qty,
                price: lock.eur_quantity,
            };
            self.notify(event);

            log_for_buy(token);
            Ok(splitted_good)
        } else {
            locked_meta.unlock_for_buy();
            log_for_buy_err(token.clone());
            // Wasn't able to split the good
            Err(BuyError::InsufficientGoodQuantity {
                contained_quantity: cash.get_qty(),
                pre_agreed_quantity: lock.locked_original_qty,
            })
        }
    }

    fn lock_sell(
        &mut self,
        kind_to_sell: GoodKind,
        quantity_to_sell: f32,
        offer: f32,
        trader_name: String,
    ) -> Result<String, LockSellError> {
        // First check if good is available
        let (_, meta) = match self.good_storage.get_good_for_kind(&kind_to_sell) {
            Some((good, meta)) => (good, meta),
            _ => {
                log_for_lock_sell_err(trader_name, kind_to_sell, quantity_to_sell, offer);
                panic!(
                    "Good {} can't be locked for sell, because it was not found",
                    kind_to_sell
                )
            }
        };
        // Is the Good already locked?
        if let Some(lock) = meta.get_sell_lock() {
            log_for_lock_sell_err(trader_name, kind_to_sell, quantity_to_sell, offer);
            return Err(LockSellError::GoodAlreadyLocked {
                token: lock.transaction_token.clone(),
            });
        }

        // Is the number of max. locks achieved
        if !self.can_add_sell_lock() {
            log_for_lock_sell_err(trader_name, kind_to_sell, quantity_to_sell, offer);
            return Err(LockSellError::MaxAllowedLocksReached);
        }

        // Not locked, but is the quantity enough?
        if quantity_to_sell <= 0.0 {
            log_for_lock_sell_err(trader_name, kind_to_sell, quantity_to_sell, offer);
            return Err(LockSellError::NonPositiveQuantityToSell {
                negative_quantity_to_sell: quantity_to_sell,
            });
        }

        // Check if we have enough EUR to sell
        if offer > self.get_budget() {
            log_for_lock_sell_err(trader_name, kind_to_sell, quantity_to_sell, offer);
            return Err(LockSellError::InsufficientDefaultGoodQuantityAvailable {
                offered_good_kind: kind_to_sell,
                available_good_quantity: self.get_budget(),
                offered_good_quantity: quantity_to_sell,
            });
        }

        // Is the offer acceptable?
        let highest_acceptable_offer = match self.get_sell_price(kind_to_sell, quantity_to_sell) {
            Ok(price) => price,
            Err(_) => {
                log_for_lock_sell_err(trader_name, kind_to_sell, quantity_to_sell, offer);
                return Err(LockSellError::NonPositiveQuantityToSell {
                    negative_quantity_to_sell: quantity_to_sell,
                });
            }
        };

        if offer <= 0.0 {
            log_for_lock_sell_err(trader_name, kind_to_sell, quantity_to_sell, offer);
            return Err(LockSellError::NonPositiveOffer {
                negative_offer: offer,
            });
        } else if offer > highest_acceptable_offer {
            log_for_lock_sell_err(trader_name, kind_to_sell, quantity_to_sell, offer);
            return Err(LockSellError::OfferTooHigh {
                offered_good_kind: kind_to_sell,
                offered_good_quantity: quantity_to_sell,
                high_offer: offer,
                highest_acceptable_offer,
            });
        }

        // get mut ref of good
        let (good, meta) = self
            .good_storage
            .get_mut_good_for_kind(&kind_to_sell)
            .expect("Could not get mut ref of good to lock for sell");

        // Lock the Good (update its metadata)
        let token = meta.lock_for_sell(quantity_to_sell, kind_to_sell, offer, trader_name.clone());

        // fluctuation
        let new_quantity = good.get_qty() - quantity_to_sell;
        let factor = good.get_qty() / new_quantity;
        meta.fluctuate_sell_price_with_factor(factor);

        // notify
        let event = Event {
            kind: EventKind::LockedSell,
            good_kind: kind_to_sell,
            quantity: quantity_to_sell,
            price: offer,
        };
        self.notify(event);

        log_for_lock_sell(
            trader_name,
            kind_to_sell,
            quantity_to_sell,
            offer,
            token.clone(),
        );
        Ok(token)
    }

    /// Call when a trader **sells to our market**
    fn sell(&mut self, token: String, good: &mut Good) -> Result<Good, SellError> {
        // get mut ref for locked good
        let (locked_good, locked_meta) = match self.good_storage.get_mut_good_for_sell_token(&token)
        {
            Some((good, meta)) => (good, meta),
            _ => {
                return if self.good_storage.has_good_expired_sell_token(&token) {
                    log_for_sell_err(token.clone());
                    Err(SellError::ExpiredToken {
                        expired_token: token,
                    })
                } else {
                    // this token is invalid
                    log_for_sell_err(token.clone());
                    Err(SellError::UnrecognizedToken {
                        unrecognized_token: token,
                    })
                };
            }
        };

        // get the lock
        let lock = locked_meta
            .get_sell_lock()
            .expect("Can't get lock for good")
            .clone();

        // check if lock is expired

        // token is valid, is the kind correct?
        if locked_good.get_kind() != good.get_kind() {
            locked_meta.unlock_for_sell();
            log_for_sell_err(token.clone());
            return Err(SellError::WrongGoodKind {
                wrong_good_kind: good.get_kind(),
                pre_agreed_kind: locked_good.get_kind(),
            });
        }

        // is the quantity correct?
        if good.get_qty() < lock.locked_original_qty {
            locked_meta.unlock_for_sell();
            log_for_sell_err(token.clone());
            return Err(SellError::InsufficientGoodQuantity {
                contained_quantity: good.get_qty(),
                pre_agreed_quantity: lock.locked_original_qty,
            });
        }

        // save old quantity
        let old_quantity = locked_good.get_qty();

        // split from sellers good
        if let Ok(remaining) = good.split(lock.locked_original_qty) {
            // Merge our good with the sold quantity (increase our capacity)
            let _ = locked_good.merge(remaining);
            // fluctuation
            let factor = old_quantity / locked_good.get_qty();
            locked_meta.fluctuate_sell_price_with_factor(factor);

            // unlock good
            locked_meta.unlock_for_sell();

            // split our default good (remove EUR from our market)
            self.split_default_good(lock.eur_quantity);

            // notify the market
            let event = Event {
                kind: EventKind::Sold,
                good_kind: lock.kind,
                quantity: lock.locked_original_qty,
                price: lock.eur_quantity,
            };
            self.notify(event);

            log_for_sell(token);
            // return the default good with the pre-agree quantity
            let res = Good::new(GoodKind::EUR, lock.eur_quantity);
            Ok(res)
        } else {
            locked_meta.unlock_for_sell();
            log_for_sell_err(token.clone());
            // Splitting wasn't successful
            Err(SellError::InsufficientGoodQuantity {
                contained_quantity: good.get_qty(),
                pre_agreed_quantity: lock.locked_original_qty,
            })
        }
    }
}
