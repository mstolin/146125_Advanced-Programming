#![allow(non_snake_case)]
#[cfg(test)]
mod test {
    use crate::market::sgx::SGX;
    use unitn_market_2022::market::market_test;

    #[test]
    fn test_name() {
        market_test::test_name::<SGX>();
    }
    #[test]
    fn test_get_buy_price_success() {
        market_test::test_get_buy_price_success::<SGX>();
    }
    #[test]
    fn test_get_buy_price_insufficient_qty_error() {
        market_test::test_get_buy_price_insufficient_qty_error::<SGX>();
    }
    #[test]
    fn test_get_buy_price_non_positive_error() {
        market_test::test_get_buy_price_non_positive_error::<SGX>();
    }
    #[test]
    fn test_get_sell_price_success() {
        market_test::test_get_sell_price_success::<SGX>();
    }
    #[test]
    fn test_get_sell_price_non_positive_error() {
        market_test::test_get_sell_price_non_positive_error::<SGX>();
    }
    #[test]
    fn test_deadlock_prevention() {
        market_test::test_deadlock_prevention::<SGX>();
    }
    #[test]
    fn test_new_random() {
        market_test::test_new_random::<SGX>();
    }
    #[test]
    fn test_price_change_after_buy() {
        market_test::test_price_change_after_buy::<SGX>();
    }
    #[test]
    fn price_changes_waiting() {
        market_test::price_changes_waiting::<SGX>();
    }
    #[test]
    fn test_price_change_after_sell() {
        market_test::test_price_change_after_sell::<SGX>();
    }
    #[test]
    fn should_initialize_with_right_quantity() {
        market_test::should_initialize_with_right_quantity::<SGX>();
    }
    #[test]
    fn new_random_should_not_exceed_starting_capital() {
        market_test::new_random_should_not_exceeed_starting_capital::<SGX>();
    }
    #[test]
    fn test_sell_success() {
        market_test::test_sell_success::<SGX>();
    }
    #[test]
    fn test_sell_err_unrecognized_token() {
        market_test::test_sell_unrecognized_token::<SGX>();
    }
    #[test]
    fn test_sell_err_expired_token() {
        market_test::test_sell_expired_token::<SGX>();
    }
    #[test]
    fn test_sell_err_wrong_good_kind() {
        market_test::test_sell_wrong_good_kind::<SGX>();
    }
    #[test]
    fn test_sell_err_insufficient_good_quantity() {
        market_test::test_sell_insufficient_good_quantity::<SGX>();
    }
    #[test]
    fn test_lock_sell_nonPositiveOffer() {
        market_test::test_lock_sell_nonPositiveOffer::<SGX>();
    }
    #[test]
    fn test_lock_sell_defaultGoodAlreadyLocked() {
        market_test::test_lock_sell_defaultGoodAlreadyLocked::<SGX>();
    }
    #[test]
    fn test_lock_sell_insufficientDefaultGoodQuantityAvailable() {
        market_test::test_lock_sell_insufficientDefaultGoodQuantityAvailable::<SGX>();
    }
    #[test]
    fn test_lock_sell_offerTooHigh() {
        market_test::test_lock_sell_offerTooHigh::<SGX>();
    }
    #[test]
    fn test_working_function_lock_sell_token() {
        market_test::test_working_function_lock_sell_token::<SGX>();
    }
    #[test]
    fn test_lock_buy_non_positive_quantity_to_buy() {
        market_test::test_lock_buy_non_positive_quantity_to_buy::<SGX>();
    }
    #[test]
    fn test_lock_buy_non_positive_bid() {
        market_test::test_lock_buy_non_positive_bid::<SGX>();
    }
    #[test]
    fn test_lock_buy_insufficient_good_quantity_available() {
        market_test::test_lock_buy_insufficient_good_quantity_available::<SGX>();
    }
    #[test]
    fn test_lock_buy_bid_too_low() {
        market_test::test_lock_buy_bid_too_low::<SGX>();
    }
    #[test]
    fn test_buy_unrecognized_token() {
        market_test::test_lock_buy_bid_too_low::<SGX>();
    }
    #[test]
    fn test_buy_good_kind_not_default() {
        market_test::test_buy_good_kind_not_default::<SGX>();
    }
    #[test]
    fn test_buy_insufficient_good_quantity() {
        market_test::test_buy_insufficient_good_quantity::<SGX>();
    }
    #[test]
    fn test_buy_success() {
        market_test::test_buy_success::<SGX>();
    }
    #[test]
    fn test_price_increase() {
        market_test::test_price_increase::<SGX>();
    }
    #[test]
    fn test_get_budget() {
        market_test::test_get_budget::<SGX>();
    }
    #[test]
    fn test_get_goods() {
        market_test::test_get_goods::<SGX>();
    }
    #[test]
    fn test_get_name() {
        market_test::test_get_name::<SGX>();
    }
}
