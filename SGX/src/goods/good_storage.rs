use crate::goods::good_metadata::GoodMetadata;
use crate::goods::good_status::GoodStatus;
use crate::goods::goods_factory::{GoodWithMeta, GoodsFactory};
use std::slice::IterMut;
use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;
use unitn_market_2022::market::good_label::GoodLabel;

/**
 * The `GoodStorage` is a wrapper around the type `Vec<GoodWithMeta>`.
 * It's purpose is, to simplify the `SGX` implementation by moving all goods related operations
 * to this struct.
 * It does only implement CRUD operations around the goods.
 */
pub struct GoodStorage(Vec<GoodWithMeta>);

// Struct functions
impl GoodStorage {
    /// Generates goods with random quantities up to the given available quantity
    pub fn new_random(available_quantity: f32) -> Self {
        Self(GoodsFactory::random_goods(available_quantity))
    }

    /// Generates goods with the given quantities
    pub fn with_quantities(eur: f32, yen: f32, usd: f32, yuan: f32) -> Self {
        Self(GoodsFactory::all_with_quantities(eur, yen, usd, yuan))
    }
}

impl GoodStorage {
    /// Generates a `GoodLabel` from the given `Good` and `GoodMetadata`
    fn get_good_label_from_good(&self, good: &Good, meta: &GoodMetadata) -> GoodLabel {
        GoodLabel {
            good_kind: good.get_kind(),
            quantity: good.get_qty(),
            exchange_rate_buy: meta.base_buy_price,
            exchange_rate_sell: meta.base_sell_price,
        }
    }

    /// Returns the len of goods
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns all goods as `GoodLabel` instances
    pub fn get_good_labels(&self) -> Vec<GoodLabel> {
        self.0
            .iter()
            .map(|(g, m)| self.get_good_label_from_good(g, m))
            .collect::<Vec<GoodLabel>>()
    }

    /// Returns the numbers of goods that are locked for buy
    pub fn get_buy_locks_len(&self) -> usize {
        self.0.iter().filter(|(_, m)| m.is_locked_for_buy()).count()
    }

    /// Returns the numbers of goods that are locked for sell
    pub fn get_sell_locks_len(&self) -> usize {
        self.0
            .iter()
            .filter(|(_, m)| m.is_locked_for_sell())
            .count()
    }

    /// Returns a mutable good reference for the given kind
    pub fn get_mut_good_for_kind(&mut self, kind: &GoodKind) -> Option<&mut GoodWithMeta> {
        self.0.iter_mut().find(|(g, _)| g.get_kind() == *kind)
    }

    /// Returns a good reference for the given kind
    pub fn get_good_for_kind(&self, kind: &GoodKind) -> Option<&GoodWithMeta> {
        self.0.iter().find(|(g, _)| g.get_kind() == *kind)
    }

    /// Returns a mutable reference to the good for the given token, if available
    pub fn get_mut_good_for_sell_token(&mut self, token: &String) -> Option<&mut GoodWithMeta> {
        self.0.iter_mut().find(|(_, m)| match &m.sell_status {
            GoodStatus::Available => false,
            GoodStatus::Locked(lock) => lock.transaction_token == *token,
        })
    }

    /// Checks if any good contains the expired sell token
    pub fn has_good_expired_sell_token(&self, token: &String) -> bool {
        self.0
            .iter()
            .any(|(_, meta)| meta.has_expired_sell_token(token))
    }

    /// Returns a mutable reference to the good for the given token, if available
    pub fn get_mut_good_for_buy_token(&mut self, token: &String) -> Option<&mut GoodWithMeta> {
        self.0.iter_mut().find(|(_, m)| match &m.buy_status {
            GoodStatus::Available => false,
            GoodStatus::Locked(lock) => lock.transaction_token == *token,
        })
    }

    /// Checks if any good contains the expired buy token
    pub fn has_good_expired_buy_token(&self, token: &String) -> bool {
        self.0
            .iter()
            .any(|(_, meta)| meta.has_expired_buy_token(token))
    }

    /// Returns the default good
    pub fn get_default_good(&self) -> &GoodWithMeta {
        self.0
            .iter()
            .find(|(g, _)| g.get_kind() == DEFAULT_GOOD_KIND)
            .expect("Not able to get default good ") // we can expect, default good should always be present
    }

    /// Returns a mut iterator of the goods
    pub fn iter_mut(&mut self) -> IterMut<GoodWithMeta> {
        self.0.iter_mut()
    }
}

#[cfg(test)]
mod tests {
    use crate::goods::good_storage::GoodStorage;
    use crate::goods::goods_factory::GoodsFactory;
    use unitn_market_2022::good::consts::DEFAULT_GOOD_KIND;
    use unitn_market_2022::good::good_kind::GoodKind;

    #[test]
    fn test_random_good_storage() {
        let storage = GoodStorage::new_random(2500.0);
        assert_eq!(
            4,
            storage.0.len(),
            "Length of random goods should be equal to 4"
        );
    }

    #[test]
    fn test_good_storage_with_quantities() {
        let storage = GoodStorage::with_quantities(5.0, 5.0, 5.0, 5.0);
        let goods = GoodsFactory::all_with_quantities(5.0, 5.0, 5.0, 5.0);
        assert_eq!(
            goods, storage.0,
            "Storage goods must be equal to the one generated by GoodsFactory"
        );
    }

    #[test]
    fn test_good_storage_len() {
        let storage = GoodStorage::with_quantities(5.0, 5.0, 5.0, 5.0);
        assert_eq!(4, storage.len(), "Storage goods len must be equal to 4");
    }

    #[test]
    fn test_get_good_labels() {
        let storage = GoodStorage::with_quantities(5.0, 5.0, 5.0, 5.0);
        let labels = storage.get_good_labels();

        assert_eq!(
            storage.len(),
            labels.len(),
            "Labels len must be equal to storage goods len of {}",
            storage.len()
        );

        for ((good, _), label) in storage.0.iter().zip(labels.iter()) {
            assert_eq!(
                label.good_kind,
                good.get_kind(),
                "Label kind has to be equal to {}",
                label.good_kind
            );
            assert_eq!(
                label.quantity,
                good.get_qty(),
                "Label quantity has to be equal to {}",
                label.quantity
            );
        }
    }

    #[test]
    fn test_get_good_for_kind() {
        let mut storage = GoodStorage::with_quantities(5.0, 5.0, 5.0, 5.0);
        let kind = GoodKind::EUR;
        let (eur, eur_meta) = storage.get_mut_good_for_kind(&kind).unwrap();
        assert_eq!(GoodKind::EUR, eur.get_kind(), "Good has to be of kind EUR");
        assert_eq!(5.0, eur.get_qty(), "Quantity if EUR has to be 5.0");
        assert!(!eur_meta.is_locked_for_buy(), "EUR is not locked for buy");
        assert!(!eur_meta.is_locked_for_sell(), "EUR is not locked for sell");
    }

    #[test]
    fn test_get_default_good() {
        let storage = GoodStorage::with_quantities(5.0, 5.0, 5.0, 5.0);
        let (default, _) = storage.get_default_good();
        assert_eq!(
            DEFAULT_GOOD_KIND,
            default.get_kind(),
            "Default good is of kind {}",
            DEFAULT_GOOD_KIND
        );
        assert_eq!(5.0, default.get_qty(), "Default good quantity is {}", 5.0);
    }

    #[test]
    fn test_buy_locks() {
        let mut storage = GoodStorage::with_quantities(5.0, 5.0, 5.0, 5.0);
        assert_eq!(
            0,
            storage.get_buy_locks_len(),
            "Lock len must be 0, no Good locked yet"
        );

        // try to get a good for an invalid token
        let invalid_token = "INVALID_TOKEN".to_string();
        let invalid_good = storage.get_mut_good_for_buy_token(&invalid_token);
        assert_eq!(
            None, invalid_good,
            "No locked good found for invalid token '{}'",
            invalid_token
        );
        assert!(
            !storage.has_good_expired_buy_token(&invalid_token),
            "Invalid '{}' token can't be expired",
            invalid_token
        );

        let (_, first_meta) = storage.0.get_mut(0).unwrap();
        let trader_name = "Test_Trader".to_string();

        // lock the EUR good
        let token = first_meta.lock_for_buy(2.0, GoodKind::EUR, 200.0, trader_name);
        assert_eq!(1, storage.get_buy_locks_len(), "One good is locked for buy");
        assert!(
            !storage.has_good_expired_buy_token(&token),
            "Valid token '{}' token can't be expired",
            token
        );

        // get locked good
        let (_, locked_meta) = storage.get_mut_good_for_buy_token(&token).unwrap();
        assert!(
            locked_meta.is_locked_for_buy(),
            "Good must be locked for buy"
        );
        assert!(
            !locked_meta.is_locked_for_sell(),
            "Good can't be locked for sell"
        );

        // unlock
        locked_meta.unlock_for_buy();
        assert!(
            !locked_meta.is_locked_for_buy(),
            "Good should not locked for buy anymore"
        );
        assert!(
            storage.has_good_expired_buy_token(&token),
            "Token '{}' token must have expired after unlock",
            token
        );
    }

    #[test]
    fn test_sell_locks() {
        let mut storage = GoodStorage::with_quantities(5.0, 5.0, 5.0, 5.0);
        assert_eq!(
            0,
            storage.get_sell_locks_len(),
            "Lock len must be 0, no Good locked yet"
        );

        // try to get a good for an invalid token
        let invalid_token = "INVALID_TOKEN".to_string();
        let invalid_good = storage.get_mut_good_for_sell_token(&invalid_token);
        assert_eq!(
            None, invalid_good,
            "No locked good found for invalid token '{}'",
            invalid_token
        );

        assert!(
            !storage.has_good_expired_sell_token(&invalid_token),
            "Invalid '{}' token can't be expired",
            invalid_token
        );

        let (_, first_meta) = storage.0.get_mut(0).unwrap();
        let trader_name = "Test_Trader".to_string();

        // lock the EUR good
        let token = first_meta.lock_for_sell(2.0, GoodKind::EUR, 200.0, trader_name);
        assert_eq!(
            1,
            storage.get_sell_locks_len(),
            "One good is locked for sell"
        );
        assert!(
            !storage.has_good_expired_sell_token(&token),
            "Valid token '{}' token can't have expired",
            token
        );

        // get locked good
        let (_, locked_meta) = storage.get_mut_good_for_sell_token(&token).unwrap();
        assert!(
            !locked_meta.is_locked_for_buy(),
            "Good should not be locked for buy"
        );
        assert!(
            locked_meta.is_locked_for_sell(),
            "Good should be locked for sell"
        );

        // unlock
        locked_meta.unlock_for_sell();
        assert!(
            !locked_meta.is_locked_for_sell(),
            "Good should not be locked for sell anymore"
        );
        assert!(
            storage.has_good_expired_sell_token(&token),
            "Token '{}' token must be expired after unlock",
            token
        );
    }
}
