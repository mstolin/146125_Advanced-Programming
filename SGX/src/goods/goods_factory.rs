use crate::goods::good_metadata::GoodMetadata;
use rand::Rng;
use unitn_market_2022::good::consts::{
    DEFAULT_EUR_USD_EXCHANGE_RATE, DEFAULT_EUR_YEN_EXCHANGE_RATE, DEFAULT_EUR_YUAN_EXCHANGE_RATE,
};
use unitn_market_2022::good::good::Good;
use unitn_market_2022::good::good_kind::GoodKind;

pub type GoodWithMeta = (Good, GoodMetadata);

/**
 * The `GoodsFactory` is a helper class that intents to generate
 * some `Good`s. The purpose of this class is, that it only cares
 * about the generation of goods, to keep the `SGX` market small
 * and remove the complexity from that.
 */
pub struct GoodsFactory();

impl GoodsFactory {
    /// Generate a vec of random quantities that sum up to the available quantity
    fn random_quantities(num: u32, mut available_quantity: f32) -> Vec<f32> {
        let mut quantities = Vec::new();
        let mut steps = 0;
        let mut rng = rand::thread_rng();

        // remove random values from quantity and add it to the vec
        while available_quantity > 0.0 && steps < (num - 1) {
            let quantity = available_quantity - rng.gen_range(0.0..available_quantity);
            available_quantity -= quantity;
            quantities.push(quantity);
            steps += 1;
        }

        // add the remaining quantity
        quantities.push(available_quantity);

        quantities
    }

    /// Returns Goods containing only an EUR good
    pub fn random_goods(available_quantity: f32) -> Vec<GoodWithMeta> {
        let random_quantities = GoodsFactory::random_quantities(4, available_quantity);
        GoodsFactory::all_with_quantities(
            random_quantities[0],
            random_quantities[1],
            random_quantities[2],
            random_quantities[3],
        )
    }

    pub fn all_with_quantities(
        eur: f32,
        yen: f32,
        usd: f32,
        yuan: f32,
    ) -> Vec<(Good, GoodMetadata)> {
        Vec::from([
            (Good::new(GoodKind::EUR, eur), GoodMetadata::new(1.0)),
            (
                Good::new(GoodKind::YEN, yen),
                GoodMetadata::new(DEFAULT_EUR_YEN_EXCHANGE_RATE),
            ),
            (
                Good::new(GoodKind::USD, usd),
                GoodMetadata::new(DEFAULT_EUR_USD_EXCHANGE_RATE),
            ),
            (
                Good::new(GoodKind::YUAN, yuan),
                GoodMetadata::new(DEFAULT_EUR_YUAN_EXCHANGE_RATE),
            ),
        ])
    }
}

#[cfg(test)]
mod tests {
    use crate::goods::goods_factory::GoodsFactory;

    #[test]
    fn test_random_goods() {
        let first_random_goods = GoodsFactory::random_goods(2500.0);
        let second_random_goods = GoodsFactory::random_goods(5000.0);
        assert_eq!(
            4,
            first_random_goods.len(),
            "Random goods must be 4 random goods"
        );
        assert_eq!(
            4,
            second_random_goods.len(),
            "Random goods must be 4 random goods"
        );

        assert_ne!(
            first_random_goods, second_random_goods,
            "Random goods should not be equal"
        );
    }

    #[test]
    fn test_with_quantities() {
        let goods = GoodsFactory::all_with_quantities(50.0, 60.0, 70.0, 80.0);
        assert_eq!(4, goods.len(), "There must be 4 goods");
        assert_eq!(
            50.0,
            goods[0].0.get_qty(),
            "First good must have a quantity of 50.0"
        );
        assert_eq!(
            60.0,
            goods[1].0.get_qty(),
            "Second good must have a quantity of 60.0"
        );
        assert_eq!(
            70.0,
            goods[2].0.get_qty(),
            "Third good must have a quantity of 70.0"
        );
        assert_eq!(
            80.0,
            goods[3].0.get_qty(),
            "Fourth good must have a quantity of 80.0"
        );
    }

    #[test]
    fn test_random_quantities() {
        let available_quantity: f32 = 1000.0;
        let random_quantities = GoodsFactory::random_quantities(4, available_quantity);
        assert_eq!(
            4,
            random_quantities.len(),
            "There must be 4 random quantities"
        );

        let sum = random_quantities
            .iter()
            .copied()
            .reduce(|a, b| a + b)
            .expect("Not able to sum up random quantities");
        assert_eq!(
            1000.0, sum,
            "The sum of all quantities must be equal to {}",
            available_quantity
        );
    }
}
