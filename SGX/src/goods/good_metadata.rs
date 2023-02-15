use crate::goods::good_status::{GoodLock, GoodStatus};
use unitn_market_2022::good::good_kind::GoodKind;

/**
 * The `GoodMetadata` struct represents meta information about the `Good`.
 */
#[derive(Debug, PartialEq, Clone)]
pub struct GoodMetadata {
    /// Base sell price of the good, in EUR
    pub base_sell_price: f32,
    /// Base buy price of the good, in EUR
    pub base_buy_price: f32,
    /// Is the good locked for sell
    pub sell_status: GoodStatus,
    /// Is the good locked for buy
    pub buy_status: GoodStatus,
    /// All expired sell tokens
    pub expired_sell_tokens: Vec<String>,
    /// All expired buy tokens
    pub expired_buy_tokens: Vec<String>,
}

impl GoodMetadata {
    /// Constructs a new `GoodMetadata` instance
    pub fn new(exchange_rate: f32) -> Self {
        Self {
            base_sell_price: 1.0 / exchange_rate,
            base_buy_price: exchange_rate,
            sell_status: GoodStatus::Available,
            buy_status: GoodStatus::Available,
            expired_sell_tokens: Vec::new(),
            expired_buy_tokens: Vec::new(),
        }
    }
}

impl GoodMetadata {
    /// Locks the Good (adds a `GoodStatus::Locked` to the metadata) for sell
    pub fn lock_for_sell(
        &mut self,
        locked_qty: f32,
        kind: GoodKind,
        offer: f32,
        trader_name: String,
    ) -> String {
        let lock = GoodLock::new(locked_qty, kind, offer, trader_name);
        let token = lock.transaction_token.clone();
        self.sell_status = GoodStatus::Locked(lock);
        token
    }

    /// Locks the Good (adds a `GoodStatus::Locked` to the metadata) for buy
    pub fn lock_for_buy(
        &mut self,
        locked_qty: f32,
        kind: GoodKind,
        bid: f32,
        trader_name: String,
    ) -> String {
        let lock = GoodLock::new(locked_qty, kind, bid, trader_name);
        let token = lock.transaction_token.clone();
        self.buy_status = GoodStatus::Locked(lock);
        token
    }

    /// Unlocks the good for sell
    pub fn unlock_for_sell(&mut self) {
        if let GoodStatus::Locked(lock) = &self.sell_status {
            self.expired_sell_tokens
                .push(lock.transaction_token.clone());
        } else {
            panic!("Cannot unlock sell lock because there is no lock");
        }
        self.sell_status = GoodStatus::Available;
    }

    /// Unlocks the good for buy
    pub fn unlock_for_buy(&mut self) {
        if let GoodStatus::Locked(lock) = &self.buy_status {
            self.expired_buy_tokens.push(lock.transaction_token.clone());
        } else {
            panic!("Cannot unlock buy lock because there is no lock");
        }
        self.buy_status = GoodStatus::Available;
    }

    /// Check if this Good is locked for sell
    pub fn is_locked_for_sell(&self) -> bool {
        match self.sell_status {
            GoodStatus::Available => false,
            GoodStatus::Locked(_) => true,
        }
    }

    /// Check if this Good is locked for buy
    pub fn is_locked_for_buy(&self) -> bool {
        match self.buy_status {
            GoodStatus::Available => false,
            GoodStatus::Locked(_) => true,
        }
    }

    /// Returns a reference of the sell lock, if available
    pub fn get_sell_lock(&self) -> Option<&GoodLock> {
        match &self.sell_status {
            GoodStatus::Available => None,
            GoodStatus::Locked(lock) => Some(lock),
        }
    }

    /// Returns a mut reference of the sell lock, if available
    pub fn get_mut_sell_lock(&mut self) -> Option<&mut GoodLock> {
        match &mut self.sell_status {
            GoodStatus::Available => None,
            GoodStatus::Locked(lock) => Some(lock),
        }
    }

    /// Returns a reference of the buy lock, if available
    pub fn get_buy_lock(&self) -> Option<&GoodLock> {
        match &self.buy_status {
            GoodStatus::Available => None,
            GoodStatus::Locked(lock) => Some(lock),
        }
    }

    /// Returns a mut reference of the buy lock, if available
    pub fn get_mut_buy_lock(&mut self) -> Option<&mut GoodLock> {
        match &mut self.buy_status {
            GoodStatus::Available => None,
            GoodStatus::Locked(lock) => Some(lock),
        }
    }

    /// Checks if it contains an expired sell token
    pub fn has_expired_sell_token(&self, token: &String) -> bool {
        self.expired_sell_tokens.contains(token)
    }

    /// Checks if it contains an expired buy token
    pub fn has_expired_buy_token(&self, token: &String) -> bool {
        self.expired_buy_tokens.contains(token)
    }

    pub fn fluctuate_buy_price_with_factor(&mut self, factor: f32) {
        self.base_buy_price *= factor;
    }

    pub fn fluctuate_sell_price_with_factor(&mut self, factor: f32) {
        self.base_sell_price *= factor;
    }
}

#[cfg(test)]
mod tests {
    use crate::goods::good_metadata::GoodMetadata;
    use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
    use unitn_market_2022::good::good_kind::GoodKind;

    #[test]
    fn test_new_good_metadata() {
        let meta = GoodMetadata::new(1.0);
        assert_eq!(1.0, meta.base_buy_price, "Base buy price must be {}", 1.0);
        assert_eq!(
            1.0 / meta.base_buy_price,
            meta.base_sell_price,
            "Base sell price must be {}",
            1.0
        );
        assert!(
            !meta.is_locked_for_sell(),
            "Metadata is not suppose to be sell-locked"
        );
        assert!(
            !meta.is_locked_for_buy(),
            "Metadata is not suppose to be buy-locked"
        );
    }

    #[test]
    fn test_lock_for_buy() {
        let trader_name = "TEST-TRADER".to_string();
        let mut meta = GoodMetadata::new(1.0);
        assert!(
            !meta.is_locked_for_sell(),
            "Metadata is not suppose to be sell-locked"
        );
        assert!(
            !meta.is_locked_for_buy(),
            "Metadata is not suppose to be buy-locked"
        );

        let invalid_buy_lock = meta.get_buy_lock();
        assert_eq!(None, invalid_buy_lock, "Buy lock must be None");
        let invalid_sell_lock = meta.get_sell_lock();
        assert_eq!(None, invalid_sell_lock, "Sell lock must be None");

        let _ = meta.lock_for_buy(100.0, GoodKind::EUR, 120.0, trader_name);
        assert!(meta.is_locked_for_buy(), "Metadata must be locked for buy");
        assert!(
            !meta.is_locked_for_sell(),
            "Metadata should not be locked for sell"
        );

        let lock = meta.get_buy_lock();
        assert!(lock.is_some(), "Buy lock can't be None");
    }

    #[test]
    fn test_lock_for_sell() {
        let trader_name = "TEST-TRADER".to_string();
        let mut meta = GoodMetadata::new(1.0);
        assert!(
            !meta.is_locked_for_sell(),
            "Metadata is not suppose to be sell-locked"
        );
        assert!(
            !meta.is_locked_for_buy(),
            "Metadata is not suppose to be buy-locked"
        );

        let invalid_buy_lock = meta.get_buy_lock();
        assert_eq!(None, invalid_buy_lock, "Buy lock must be None");
        let invalid_sell_lock = meta.get_sell_lock();
        assert_eq!(None, invalid_sell_lock, "Sell lock must be None");

        let _ = meta.lock_for_sell(100.0, GoodKind::EUR, 120.0, trader_name);
        assert!(meta.is_locked_for_sell(), "Metadata must be sell-locked");
        assert!(
            !meta.is_locked_for_buy(),
            "Metadata is not suppose to be buy-locked"
        );

        let lock = meta.get_sell_lock();
        assert!(lock.is_some(), "Sell lock can't be None");
    }

    #[test]
    fn test_fluctuation() {
        let mut meta = GoodMetadata::new(1.0);

        // test buy fluctuation
        let old_quantity = 1000.0;
        let new_quantity = 500.0;
        let old_buy_price = meta.base_buy_price;
        meta.fluctuate_buy_price_with_factor(old_quantity / new_quantity);
        assert!(
            old_buy_price < meta.base_buy_price,
            "New buy price has not increased. Old: {}, new: {}",
            old_buy_price,
            meta.base_buy_price
        );

        // test buy fluctuation
        let old_quantity = 500.0;
        let new_quantity = 1000.0;
        let old_sell_price = meta.base_sell_price;
        meta.fluctuate_sell_price_with_factor(old_quantity / new_quantity);
        assert!(
            old_sell_price > meta.base_sell_price,
            "New sell price has not decreased. Old: {}, new: {}",
            old_sell_price,
            meta.base_sell_price
        );
    }

    #[test]
    fn test_expired_tokens() {
        let trader_name = "TEST_TRADER".to_string();
        let mut meta = GoodMetadata::new(1.0);
        assert!(
            meta.expired_sell_tokens.is_empty(),
            "There can't be any expired sell token after creation"
        );
        assert!(
            meta.expired_buy_tokens.is_empty(),
            "There can't be any expired buy token after creation"
        );

        let false_token = "FALSE_TOKEN".to_string();
        assert!(
            !meta.has_expired_sell_token(&false_token),
            "Sell-Token '{}' can't be expired because it does not exist",
            false_token
        );
        assert!(
            !meta.has_expired_buy_token(&false_token),
            "Buy-Token '{}' can't be expired because it does not exist",
            false_token
        );

        // test sell
        let token = meta.lock_for_sell(100.0, DEFAULT_GOOD_KIND, 100.0, trader_name.clone());
        assert!(
            meta.expired_sell_tokens.is_empty(),
            "There can't be any expired sell-token (None has expired yet)"
        );
        assert!(
            !meta.has_expired_sell_token(&token),
            "Sell-Token '{}' can't have expired yet",
            token
        );
        meta.unlock_for_sell();
        assert!(
            !meta.expired_sell_tokens.is_empty(),
            "There must be at least one expired sell-token"
        );
        assert!(
            meta.has_expired_sell_token(&token),
            "Sell-Token '{}' must be expired",
            token
        );

        // test buy
        let token = meta.lock_for_buy(100.0, DEFAULT_GOOD_KIND, 100.0, trader_name);
        assert!(
            meta.expired_buy_tokens.is_empty(),
            "There can't be any expired buy-token (None has expired yet)"
        );
        assert!(
            !meta.has_expired_buy_token(&token),
            "Buy-Token '{}' can't be expired yet",
            token
        );
        meta.unlock_for_buy();
        assert!(
            !meta.expired_buy_tokens.is_empty(),
            "There must be at least one expired buy-token"
        );
        assert!(
            meta.has_expired_buy_token(&token),
            "Buy-Token '{}' must have expired",
            token
        );
    }
}
