# Trader

This is the library containing all the code of running a trader
agent on certain markets.

## Usage

```rust
let mut sgx = SGX::new_random();
let markets = Vec::from([sgx]);

let trader = Trader::from(
    StrategyIdentifier::BuyAndHold,
    300_000.0, // starting capital in EUR
    markets
);

while trader.get_days() < 24 { // run trader for 24 days (starts at 0)
    trader.run();
}

let history = trader.get_history();
```

### Result

The result of the trader is a vector representing the history
of its buying and selling actions. There exist 4 different
goods (EUR, USD, YEN, YUAN - in alphabetical order) that are
tradeable. The trader starts with each good at quantity 0,
except for EUR that contains the starting capital initially.
After each day, a row (representing a day) is added to the 
vector containing the updated quantities.

For example:

```rust
[
  //  EUR  |  USD  |  YEN  |  YUAN 
    [300000,      0,      0,      0], // Initially at day 0
    [250000, 220200,   5000,    450], // After day 1
    ... // until the last day
]
```
