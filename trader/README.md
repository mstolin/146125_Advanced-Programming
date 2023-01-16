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

### Result

The result of the trader is a vector representing the history
of its buying selling actions. There exist 4 different goods
(EUR, USD, YEN, YUAN - in alphabetical order) that are
tradeable. The trader starts with each good at quantity 0,
except for EUR, which contains the starting capital. After 
each day, a row (representing a day) is added to the vector 
containing the new quantities after the day.

For example:

```rust
[
    [300000, 0, 0, 0], // Day 0 (EUR, USD, YEN, YUAN)
    [250000, 220200, 5000, 450, 15000], // Day 2
    ... // until the last day
]
```
