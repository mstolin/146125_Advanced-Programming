# Trader

This is the library containing all the code of running a trader
agent on certain markets.

## Usage

```rust
let sgx = SGX::new_random();
let markets = Vec::from([Rc::clone(&sgx)]);

let trader = Trader::from(
    StrategyIdentifier::Most_Simple,
    1_000_000.0,
    markets,
);

trader.apply_strategy(7, 30); // Run trader for 7 days, every 30 minutes

let history = trader.get_history(); // get the history for further computations
```

### How to create a new strategy

#### Step 1

Create a new file with a meaningful name at `/src/strategy/YOUR_STRATEGY.rs`. Make your
new strategy public, add it to `/src/strategy/mod.rs`, and implement the
`Strategy` trait.

```rust
use crate::strategy::strategy::Strategy;

pub struct YourNewStrategy {
    // Your custom logic here
}

impl Strategy for MostSimpleStrategy {
    fn new(markets: Vec<MarketRef>) -> Self {
        // Your custom logic here
    }

    fn get_markets(&self) -> &Vec<MarketRef> {
        // Your custom logic here
    }

    fn apply(&self, goods: &mut Vec<Good>, trader_name: &String) {
        // Your custom logic here
    }
}
```

#### Step 2

Create a new identifier for your trader by extending the enum 
`StrategyIdentifier` at `/src/trader/mod.rs`:

```rust
enum StrategyIdentifier {
    Most_Simple,
    YOUR_NEW_TRADER, // Add your new trader
}
```

#### Step 3

Define the name of your strategy at `/src/consts.rs`. The const name should
start with *TRADER_NAME_* to keep consistency. For example:

```rust
pub const TRADER_NAME_MOST_SIMPLE: &str = "TheMostSimpleTrader";
```

Then, add the name to `get_name_for_strategy` at `/src/trader/mod.rs`.
For example:

```rust
fn get_name_for_strategy(id: StrategyIdentifier) -> String {
    match id {
        StrategyIdentifier::Most_Simple => TRADER_NAME_MOST_SIMPLE.to_string(),
        StrategyIdentifier::YOUR_NEW_TRADER => TRADER_NAME_YOUR_NEW_TRADER.to_string(), // Add your new trader
    }
}
```

Lastly, at `/src/trader/mod.rs` extend the `test_get_name_for_strategy` test
case with your name:

```rust
let possible_strategies = [
    (StrategyIdentifier::Most_Simple, TRADER_NAME_MOST_SIMPLE),
    (StrategyIdentifier::YOUR_NEW_TRADER, TRADER_NAME_YOUR_NEW_TRADER), // Add your new trader
];

...
```

### History

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
