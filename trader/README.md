# Trader

This is a library containing all the code of running a trader
agent on certain markets.

## Usage

```rust
let mut sgx = SGX::new_random();
let markets = Vec::from([sgx]);

let trader = Trader::new(
    StrategyIdentifier::BuyAndHold, 
    markets
);

while trader.get_days() < 24 {
    trader.run();
}

let result = trader.get_result();
```
