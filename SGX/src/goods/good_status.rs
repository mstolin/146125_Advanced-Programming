use unitn_market_2022::good::good_kind::GoodKind;

/**
 * A `GoodStatus` identifies of a `GoodMetadata` is locked.
 * If `Available` then the metadata is not locked, otherwise `Locked(_)`
 * means the metadata is locked.
 * `Locked(GoodLock)` contains meta information about the lock.
 */
#[derive(Debug, PartialEq, Clone)]
pub enum GoodStatus {
    Locked(GoodLock),
    Available,
}

/**
 * A `GoodLock` contains meta information about a `GoodStatus`.
 * In general it represents a lock.
 */
#[derive(Debug, PartialEq, Clone)]
pub struct GoodLock {
    /// The quantity of the original good, that is locked
    pub locked_original_qty: f32,
    /// The locked kind
    pub kind: GoodKind,
    /// This is either the offer (sell) or the bid (buy), in EUR
    pub eur_quantity: f32,
    /// The token that's identifies the lock
    pub transaction_token: String,
    /// Age of the lock in days
    pub age_in_days: u8,
}

impl GoodLock {
    /// Generates a token to identify the lock
    fn gen_token(trader_name: String, kind: GoodKind, locked_qty: f32) -> String {
        format!("{}-{}-{}", trader_name, kind, locked_qty)
    }

    /// Constructs a new instance of `GoodLock`
    pub fn new(
        locked_original_qty: f32,
        kind: GoodKind,
        eur_quantity: f32,
        trader_name: String,
    ) -> Self {
        Self {
            locked_original_qty,
            kind,
            eur_quantity,
            transaction_token: GoodLock::gen_token(trader_name, kind, locked_original_qty),
            age_in_days: 1,
        }
    }
}

impl GoodLock {
    /// Increases the age by one
    pub fn increase_age_by_one(&mut self) {
        self.age_in_days += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::goods::good_status::GoodLock;
    use unitn_market_2022::good::good_kind::GoodKind;

    #[test]
    fn test_token_generation() {
        let token = GoodLock::gen_token("test".to_string(), GoodKind::EUR, 0.0);
        let expected_token = format!("{}-{}-{}", "test", GoodKind::EUR, 0.0);
        assert_eq!(
            expected_token, token,
            "Token is not equal the expected token {}",
            expected_token
        );
    }

    #[test]
    fn test_new_good_lock() {
        let locked_original_qty: f32 = 10.0;
        let kind = GoodKind::EUR;
        let eur_quantity: f32 = 200.0;
        let init_age: u8 = 1;
        let trader_name = "TEST_TRADER".to_string();
        let expected_token = format!("{}-{}-{}", trader_name, GoodKind::EUR, locked_original_qty);

        let status = GoodLock::new(locked_original_qty, kind, eur_quantity, trader_name);
        assert_eq!(
            locked_original_qty, status.locked_original_qty,
            "Status original locked quantity is not {}",
            locked_original_qty
        );
        assert_eq!(kind, status.kind, "Status kind is not {}", kind);
        assert_eq!(
            expected_token, status.transaction_token,
            "Expected token is not {}",
            expected_token
        );
        assert_eq!(
            eur_quantity, status.eur_quantity,
            "The locked eur quantity is expected to be {}",
            eur_quantity
        );
        assert_eq!(init_age, status.age_in_days, "Lock age is not {}", init_age);
    }
}
