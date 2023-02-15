# SGX

## How prices are calculated

Prices are based on demand. The higher the demand, the higher the
price and vice versa.

The demand is based on quantities. The higher the difference between
the new and the old quantity, the higher the price will be. This
will also result that prices increase when an agent buys from this
market and prices decrease when an agent sells to this market. The
goal is to always have a demand factor that is > 0 and < 1 if the
demand is low, and > 1 if the demand is high.

To make a difference between buy and sell prices, a margin of 5% is
added to the buy price and a margin of 15% is added to the sell 
price.

An example (sell price):

```rust
// Get sell price in eur for quantity
let exchange_price = quantity * exchange_rate_eur;
// Calculate a factor based on demand
let new_quantity = available_good_quantity + quantity;
let demand_factor = available_good_quantity / new_quantity;
// Calculate a margin
let margin = exchange_price * 0.15;

let price = (exchange_price + margin) * demand_factor;
```

## How prices fluctuate

Fluctuation happens at each `sell`, `buy`, `sell_lock`, `buy_lock` 
methods and extern events.

At the internal methods, the buy and sell exchange rates fluctuate 
based on demand as the price calculation. The current exchange rate
is then multiplied with a demand factor that is computed as seen 
before.

At external events (when other markets buy, sell, or lock) 
fluctuation happens as well.

Whenever a good is sold to an agent by the market, it means the 
demand of that good has increased. Therefore, the market decreases
its price for buy, slightly cheaper as the price of the buy event.
Another, point is, if an agent buys, it also wants to sell at some
point. Therefore, the market increases the sell price by 5%.

Whenever a good was sold to another market, the agent tries to make
as much profit as possible. Then, the market slightly increases its
sell price, to increase its profit. Furthermore, we increase the 
buy price as well by 5%.

## Development

- Explain GoodStorage
- Explain GoodsFactory
- Explain locking mechanism

### Configure Cargo

Refer to this documentation [https://www.bitfalter.com/documentation#ConfigureCargo](https://www.bitfalter.com/documentation#ConfigureCargo).

You need to configure the *kellnr* registry in a **global** 
`config.toml` file. The file is located under windows at 
`%USERPROFILE%\.cargo\config.toml` and under *Unix at 
`$HOME/.cargo/config.toml`.

For example:

```toml
[registries]
kellnr = { index = "git://advancedprogramming.disi.unitn.it/index", token = "yourauthtoken" }
```


### Publish to kellnr

To publish to kellnr, just run `$ cargo publish --registry kellnr`.
