# Trader

This is the library containing all the code of running a trader
agent on certain markets.

## Available Strategies

| Identifier                          | File                                                                    | Author        | Description                                            |
|-------------------------------------|-------------------------------------------------------------------------|---------------|--------------------------------------------------------|
| `StrategyIdentifier::AverageSeller` | [average_seller_strategy.rs](src/strategies/average_seller_strategy.rs) | Marcel Stolin | [AverageSellerStrategy.md](./AverageSellerStrategy.md) |

## Usage

```rust
let sgx = SGX::new_random();
let markets = vec![Rc::clone(&sgx)];

let trader = Trader::from(
    StrategyIdentifier::AverageSeller,
    1_000_000.0,
    markets,
);

trader.apply_strategy(7, 30); // Run trader for 7 days, every 30 minutes

let history = trader.get_history(); // get the history for further computations
let json = trader.get_history_as_json(); // or get the history as JSON string
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
    fn new(markets: Vec<MarketRef>, trader_name: &str) -> Self {
        // Your custom logic here
    }

    fn get_markets(&self) -> &Vec<MarketRef> {
        // Your custom logic here
    }

    fn sell_remaining_goods(&self, goods: &mut Vec<Good>) {
        // Your custom logic here
    }

    fn apply(&self, goods: &mut Vec<Good>) {
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
of its buying and selling actions. It includes the 4 different
goods (EUR, USD, YEN, YUAN - in alphabetical order) that are
tradeable. The trader starts with each good at quantity 0.0,
except for EUR that contains the starting capital initially.
After each day, a row (representing a day) is added to the 
vector containing the updated quantities.

For example:

```rust
[
    HistoryDay { day: 0, eur: 1000000.0, usd: 0.0, yen: 0.0, yuan: 0.0 },
    HistoryDay { day: 1, eur: 171150.75, usd: 20091.201, yen: 0.0, yuan: 25114.344 },
    HistoryDay { day: 2, eur: 7891.1123, usd: 20091.201, yen: 0.0, yuan: 25114.344 },
    HistoryDay { day: 3, eur: 29669.178, usd: 20091.201, yen: 0.0, yuan: 25114.344 },
    HistoryDay { day: 4, eur: 20376.188, usd: 20091.201, yen: 0.0, yuan: 25114.344 },
    HistoryDay { day: 5, eur: 11038.449, usd: 20091.201, yen: 0.0, yuan: 25114.344 },
    HistoryDay { day: 6, eur: 4425.7246, usd: 20091.201, yen: 0.0, yuan: 25114.344 },
    HistoryDay { day: 7, eur: 14692.314, usd: 20091.201, yen: 0.0, yuan: 0.0 }
]
```

It is also possible to export the history in JSON format:

```json
[
  {
    "day": 0,
    "eur": 1000000,
    "usd": 0,
    "yen": 0,
    "yuan": 0
  },
  {
    "day": 1,
    "eur": 395919.12,
    "usd": 0,
    "yen": 6106915,
    "yuan": 23970.906
  },
  {
    "day": 2,
    "eur": 1206839.5,
    "usd": 0,
    "yen": 0,
    "yuan": 23970.906
  },
  {
    "day": 3,
    "eur": 1206839.5,
    "usd": 0,
    "yen": 0,
    "yuan": 23970.906
  },
  {
    "day": 4,
    "eur": 1206839.5,
    "usd": 0,
    "yen": 0,
    "yuan": 23970.906
  },
  {
    "day": 5,
    "eur": 1206839.5,
    "usd": 0,
    "yen": 0,
    "yuan": 23970.906
  },
  {
    "day": 6,
    "eur": 1206839.5,
    "usd": 0,
    "yen": 0,
    "yuan": 23970.906
  },
  {
    "day": 7,
    "eur": 1244055,
    "usd": 0,
    "yen": 0,
    "yuan": 0
  }
]
```
