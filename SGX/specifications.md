# The Market Protocol Specifications

The purpose of this document is to define the specs of the market protocol, which involves these notions:
- goods
- goods metadata
- markets

The document then also covers
- code testing

for the aforementioned notions and concludes with a summary of the files that make the Market Protocol.
This document also refers to the trader, the other entity you have to program, as described in the Project Description.
Note that while this document is called "Market Protocol Specifications" it should also specify some important internal behaviors and structures, not only how the components communicate.

## Overall Description
The trader acquires some quantity of `goods` from each market according to the `buy` and `sell` prices that a Good has for a specific market.
The same good can exist in different markets with different `quantity`, a different `buy` price, and a different `sell` price.
When a trader has a good, the good does not have a `buy` or `sell` price.


## Good, GoodKind and GoodError
A central entity to the Market Protocol is that of goods, that exist in markets and that are traded by the trader bots.

### Good Description
`Good` is a struct with the following private fields:
- `kind : GoodKind`
- `quantity : f32`

### GoodKind Description
`GoodKind` is an Enum that defines all the possible kinds of goods that can be created, and so it contains:  
EUR, USD, YEN, YUAN


### Goods Creation
There are multiple ways to create a good.
These functions never fail; therefore, if a parameter is malformed, they initialize the corresponding field to a default
value.

```rust
pub fn new(kind: GoodKind, quantity: f32) -> Good
```
> Returns a good with the specified good kind and quantity (see below).
>
> If `quantity` is negative, the returned good has zero quantity.

```rust
pub fn new_default() -> Good
```
> Returns a good with default good kind and default quantity (see below).

"Default quantity" refers to the `DEFAULT_GOOD_STARTING_QUANTITY` constant in the `consts.rs` file.

### Goods Functionality
A Good offers the following public functions with the following signatures


```rust
pub fn split(&mut self, by_positive_quantity: f32) -> Result<Good,GoodSplitError>
``` 
> if `self.quantity` is greater than or equal to `by_positive_quantity` decrement `self.quantity` by `by_positive_quantity` and return a new Good with `quantity` = `by_positive_quantity`.
>
> **Errors**
>
> `GoodSplitError::NotEnoughQuantityToSplit` : if `by_positive_quantity` > `self.quantity`.
>
> `GoodSplitError::NonPositiveSplitQuantity` : if `by_positive_quantity` is negative.


```rust
pub fn get_qty(&mut self) -> f32  
```
> returns `self.quantity`


```rust
pub fn get_kind(&self) -> GoodKind 
```
> returns the kind of the good


```rust
pub fn merge(&self, other: Good) -> Result<(),GoodMergeError>  
```
> assuming the quantity of `other` is `q`, sets the quantity of `other` to `0` and add `q` to `self.quantity`,
>
> **Errors**
>
> `GoodMergeError::DifferentKindsOfGood(other)` : if the kind of good of self and other is different, note that it returns the ownership of the good in case of a mismatched type.


## Good Metadata
Markets must maintain a list of goods and metadata for each good as well.
An example of metadata could be:
- the base sell price
- the base buy price
- the status of the good (is looked or not), this value includes the following parameter for each good:
    - the quantity  looked
    - the quantity in exchange
    - the kind in exchange (in case of sell)
    - the kind looked (in case of buy)
    - the token of the transaction
    - the time of locking (in case of unlock related to time)

these metadata are internal to the market,
so they will not be tested directly, (so each market is free to implement them as they want)  
but they will be tested indirectly when testing all the public functions of the market

## Goods Errors
each method in the goods can return some errors, these are the following:

### GoodKindError
- `NonExistentGoodKind` returned by the 'GoodKind::from_str' function, if the string doesn't match any GoodKind

### GoodSplitError
- `NonPositiveSplitQuantity` returned by the split method, if someone calls it with a negative quantity to spit
- `NotEnoughQuantityToSplit` returned by the split method, if someone calls it with a negative quantity to spit

### GoodMergeError
- `DifferentKindsOfGood(Good)` returned by the merge method, if someone tries to merge 2 goods of different kinds. this also returns the ownership of the good, so it doesn't get deallocated

## Market
A market has a name and collection of goods, each with its own good metadata.
A market is meant to provide a buy and a sell interface, alongside other functionalities.

The list of goods is the same for all markets (i.e., for all groups).

### Market Creation
the market has 3 "constructors":

```rust
pub fn new_random() -> Rc<RefCell<dyn Market>>
```
> The market can start with the number of goods it prefers, as long as the total value of the market in
> default_good_kind, calculated using the default exchange (defined in the 'consts.rs') rate does not exceed the STARTING_CAPITAL const
> (which is 1Mln€ where € is the default_good_kind).
> To access the default exchange rate use the `get_default_exchange_rate` methods on a `GoodKind`, the methods return a
> f32 which is the exchange rate relative to the default good kind (e.g: GoodKind::USD.get_default_exchange_rate() = 1.03576, which means that 1€ = 1.03576$)


```rust
pub fn new_with_quantities(eur: f32, yen: f32, usd: f32, yuan: f32) -> Rc<RefCell<dyn Market>>
```
> Let the caller decide the quantity of each good kind. It is intended to be used for debugging and testing


```rust
pub fn new_file(path: &str) -> Rc<RefCell<dyn Market>>
```
> initialize the market using a file, if the file is not found, or has bad values in it, the market initialize with `new_random()`,
> when the instance of the market is dropped, it automatically updates the values within the file, a team can decide to not implement
> this feature and always call `new_random()` when this function is called


### Market Functionality
Markets provide some functionalities beyond the buy and sell protocols:

```rust
pub fn get_name(&self) ->  &'static str 
```
> returns the market `name`

```rust
pub fn get_budget(&self) -> f32  
```
> returns the quantity of good `EUR` of the market

```rust
pub fn get_buy_price(&self, kind: GoodKind, quantity: f32) ->  Result<f32, MarketGetterError>
```
> Return the price (in default good kind) the market wants in exchange for the quantity `quantity` of good kind `kind`.
>
> **Errors**
>
> `NonPositiveQuantityAsked` : if the quantity is not positive.
>
> `InsufficientGoodQuantityAvailable { requested_good_kind: GoodKind, requested_good_quantity: f32, available_good_quantity: f32 }` : if the quantity the trader is asking to buy is lower than the quantity the market owns.


```rust
pub fn get_sell_price(&self, kind: GoodKind, quantity: f32) -> Result<f32, MarketGetterError>
```
> Return the price (in default good kind) the market is willing to pay in exchange for the quantity `quantity` of good kind `kind`
>
> **Errors**
>
> `NonPositiveQuantityAsked` : if the quantity is not positive.


```rust
pub fn get_goods(&self) -> Vec<GoodLabel>  
```
> returns a vector containing all of the goods kinds the market sells along with their quantity and sell/buy exchange rates. See the `GoodLabel` definition below for more information. Each good kind will appear at most once in this vector.

#### Good label
`GoodLabel` is a struct containing only public fields representing the state of a good in a market:
- `good_kind : GoodKind` is the good kind this label refers to
- `quantity : f32` is the amount of Good the market has
- `exchange_rate_buy : f32` is the minimum rate at which the market accepts buyouts (see the lock-buy function for more information)
- `exchange_rate_sell : f32` is the maximum rate at which the market accepts sales (see the lock-sell function for more information)

#### MarketGetterError
the `get_sell_price` and `get_buy_price` can return the following errors:
- `NonPositiveQuantityAsked` if the quantity the trader is asking to sell or buy is not positive
- `InsufficientGoodQuantityAvailable { requested_good_kind: GoodKind, requested_good_quantity: f32, available_good_quantity: f32 }` if
  the quantity the trader is asking to buy is lower than the quantity the market owns


### Trader Buying from Market Protocol

#### The lock buy function

In order to buy a good the trader MUST lock it before, using the following function, Locking is like signing a contract: once the market accepts the lock he cannot negotiate the price or refuse the deal.

```rust
pub fn lock_buy(&mut self, kind_to_buy : GoodKind, quantity_to_buy : f32, bid : f32, trader_name : String) -> Result<String, LockBuyError>
```
> **Parameters**
>
> - `kind_to_buy` : type of good the trader want to buy
> - `quantity_to_buy` : the quantity of good the trader wants to buy
> - `bid` : the total amount of default currency the trader will give in exchange for the total amount of goods the trader will receive
> - `trader_name` : the name of the trader
>
> **Failure Cases**
>
> The Lock procedure can fail if one of the following conditions is met.
>
> - The specified good is already locked and the market allows just 1 lock per good.
> - The maximum number of different goods that can be locked at the same time has been reached.
> - The market doesn't have enough quantity available (i.e. not locked) of the requested Good.
> - The bid is below the minimum acceptable by the market.
>
> **Procedure in case of success**
>
> if no failure condition is met and there aren't any other error then the lock succeeds and the market has to:
>
> 1. register (via the market-local Good Metadata) the fact that quantity `quantity_to_buy` of good `kind_to_buy` is to be bought for price `bid`.
> 2. notify all the markets of the lock
> 3. update the price of all de goods according to the rules in the Market prices fluctuation section
> 4. return a string, which is a token that uniquely identifies the lock, the trader has to use the token when he calls the `buy` function,
     > the token system is designed to allow a market to lock the same good for different transactions multiple times, but a team is free to not implement this feature, allowing only a lock per GoodKind to be done at the same time.
>
> **Errors**
>
> 1. `LockBuyError::NonPositiveQuantityToBuy { negative_quantity_to_buy: f32 }`: returned if the quantity_to_buy is negative, it also returns the negative value back for extra clarity
> 2. `LockBuyError::NonPositiveBid { negative_bid: f32 }`: returned if the bid is negative, it also returns the negative value back for extra clarity
> 3. `LockBuyError::GoodAlreadyLocked { token: String }`: returned if a lock is already in place and the team has chosen to not allow multiples locks at the same time, this also returns the token of the already locked good
> 4. `LockBuyError::MaxAllowedLocksReached`: returned if the trader tries to lock too many Goods, and is here to prevent the deadlock (see Market Deadlock section)
> 5. `LockBuyError::InsufficientGoodQuantityAvailable { requested_good_kind: GoodKind, requested_good_quantity: f32, available_good_quantity: f32 }`: returned if the market doesn't have the quantity asked by the trader (note that your market has to keep track not only of the quantity he owns but also how much of it is already locked)
> 6. `LockBuyError::BidTooLow { requested_good_kind: GoodKind, requested_good_quantity: f32, low_bid: f32, lowest_acceptable_bid: f32 }`: returned if the bid is too low, and the deal is not valuable for the market, this function also return the minimum bid the market would accept
>
> the numeric order also set the priority of the errors, which means that if both error number 1 and number 3 occur, the market has to return error number 1.

#### The buy function

When a trader has locked a good to be bought from a market he can call the `buy` function, to move the goods and complete the deal.

```rust
pub fn buy(&mut self, token: String, cash:  &mut Good) -> Result<Good, BuyError>
```
> **Parameters**
>
> - `token`: the token given back from the lock function that uniquely identifies that lock
> - `cash`: the mutable reference of a good with default good king, with at least the pre-agreed quantity
>
> **Procedure in case of success**
>
> if no error happened then the market has to:
>
> 1. split from `cash` the quantity they agreed on and put it in the market inventory
> 2. reset the lock that was in place
> 3. notify all the markets of the transaction
> 4. update the price of all de goods according to the rules in the Market prices fluctuation section
> 5. return the pre-agreed quantity of the pre-agreed good kind
>
> **Errors**
>
> 1. `BuyError::UnrecognizedToken { unrecognized_token: String }`: returned if the token passed by the trader is not recognized, it returns the bad token for extra clarity
> 2. `BuyError::ExpiredToken { expired_token: String }`: returned if the token passed by the trader has expired (see Market Deadlock section), it returns the bad token for extra clarity
> 3. `BuyError::GoodKindNotDefault { non_default_good_kind: GoodKind }`: returned if `cash` is not the default kind, it also returns the wrong kind for extra clarity
> 4. `BuyError::InsufficientGoodQuantity { contained_quantity: f32, pre_agreed_quantity: f32 }`: returned if the quantity cash is lower than the quantity decided with the lock function, it also returns the pre-agreed quantity for extra clarity
>
> the numeric order also set the priority of the errors, which means that if both error number 1 and number 3 occur, the market has to return error number 1

### Trader Selling to Market Protocol
#### The lock sell function

In order to sell a good the trader MUST lock it before, using this function. Locking is like signing a contract: once the market accepts the lock he cannot negotiate the price or refuse the deal.

```rust
pub fn lock_sell(&mut self, kind_to_sell: GoodKind, quantity_to_sell: f32, offer: f32, trader_name: String) -> Result<String, LockSellError>
```
> **Parameters**
> - `kind_to_sell`: type of good the trader want to sell
> - `quantity_to_sell`: the quantity of good the trader wants to sell
> - `offer`: the quantity of the default good kind the trader wants in exchange for the good `kind_to_sell` with quantity `quantity_to_sell`
> - `trader_name`: the name of the trader
>
> **Failure Cases**
>
> The Lock procedure can fail if one of the following conditions is met.
>
> - The default good is locked and the market allows just 1 lock per good.
> - The maximum number of different goods that can be locked at the same time has been reached.
> - The market doesn't have enough quantity available (i.e. not locked) of the default good.
> - The offer is higher than the maximum acceptable by the market.
>
> **Procedure in case of success**
>
> if no failure condition is met and there aren't any other errors then the lock succeeds and the market has to:
>
> 1. register (via the market-local Good Metadata) the fact that quantity `quantity_to_sell` of good `kind_to_sell` is to be sold for the price `offer`.
> 2. notify all the markets of the lock
> 3. update the price of all de goods according to the rules in the Market prices fluctuation section
> 4. return a string, which is a token that uniquely identifies the lock, the trader has to use the token when he calls the `sell` function,
     >    the token system is designed to allow a market to lock the same good for different transactions multiple times, but a team is free to not implement this feature, allowing only a lock per good kind to be done at the same time.
>
> **Errors**
>
> 1. `LockBuyError::NonPositiveQuantityToSell { negative_quantity_to_sell: f32 }`: returned if the quantity_to_sel is negative, it also returns the negative value back for extra clarity
> 2. `LockBuyError::NonPositiveOffer { negative_offer: f32 }`: returned if the offer is negative, it also returns the negative value back for extra clarity
> 3. `LockBuyError::DefaultGoodAlreadyLocked { token: String }`: returned if a lock is already in place and the team has chosen to not allow multiples locks at the same time, this also returns the token of the already locked good
> 4. `LockBuyError::MaxAllowedLocksReached`: returned if the trader try to lock too many goods, and is here to prevent the deadlock (see Market Deadlock section)
> 5. `LockBuyError::InsufficientDefaultGoodQuantityAvailable  { offered_good_kind: GoodKind, offered_good_quantity: f32, available_good_quantity: f32 }`: returned if the market doesn't have the quantity asked by the trader (note that your market has to keep track not only of the quantity he owns but also how much of it is already locked)
> 6. `LockBuyError::OfferTooHigh { offered_good_kind: GoodKind, offered_good_quantity: f32, high_offer: f32, highest_acceptable_offer: f32 }`: returned if the offer is too high, and the deal is not valuable for the market, this function also return the maximum offer the market would accept
>
> the numeric order also set the priority of the errors, which means that if both error number 1 and number 3 occur, the market has to return error number 1.

#### The sell function

When a trader has locked a good to be sold from a market he can call the `sell` function, to move the goods and complete the deal.

```rust
pub fn sell(&mut self, token: String, good: &mut Good) -> Result<Good, SellError>
```
> **Parameters**
> - `good`: the good the trader is selling
> - `token`: the token given back from the lock function that uniquely identifies that lock
>
> **Procedure in case of success**
>
> if no error happened then the market has to:
>
> 1. split from `good` the quantity they agreed on and put it in the market inventory
> 2. reset the lock that was in place
> 3. notify all the markets of the transaction
> 4. update the price of all de goods according to the rules in the Market prices fluctuation section
> 5. return the pre-agreed quantity of the default good kind
>
> **Errors**
>
> 1. `SellError::UnrecognizedToken { unrecognized_token: String }`: returned if the token passed by the trader is not recognized, it returns the bad token for extra clarity
> 2. `SellError::ExpiredToken { expired_token: String }`: returned if the token passed by the trader has expired (see Market Deadlock section), it returns the bad token for extra clarity
> 3. `SellError::WrongGoodKind { wrong_good_kind: GoodKind, pre_agreed_kind: GoodKind }`: returned if `good` doesn't have the kind associated with the token, it also returns the wrong kind and the expected kind for extra clarity
> 4. `SellError::InsufficientGoodQuantity { contained_quantity: f32, pre_agreed_quantity: f32 }`: returned if the quantity cash is lower than the quantity decided with the lock function, it also returns the pre-agreed quantity for extra clarity
>
> the numeric order also set the priority of the errors, which means that if both error number 1 and number 3 occurs, the market has to return error number 1


### Market Deadlock
a market deadlock can occur if a trader intentionally or unintentionally locks all the Goods in the market,
to prevent this the market has two options:

**time unlocking**

> If a trader locks a good, and he does not buy it, after a certain time the market is allowed to unlock it.  
> The amount of time has to be in between 3 and 15 days (where a day is a call to the notify function of the observer pattern)

**lock counting**

> The market does not allow a trader to lock new goods if he already has too many goods locked.  
> The max amount must not exceed the total amount of GoodKind minus two.


Each market has to implement at least one of these two strategies.

### Buying and Selling Default Good: EUR
When buying and selling EUR (the default good used to pay for any other good), do not increase or decrease its metadata price. Buying and selling EURs is always done at a 1:1 ratio.


### Making markets reactive

Markets react to the events happening in other markets (Market events).
When a trader buys a good kind in a certain quantity from some market (a Buy event)
or sells to some market (a Sell event) or lock a good from some market (a Lock event),
other markets want to observe these events in order and react consequently.
The never-ending succession of Market events dictates the advancement of the simulation time; that is, a Market event is
a simulation tick. Upon observing a Buy or Sell event, a market applies the good generation guidelines and possibly
unlocks some goods.

This translates to the [Observer design pattern](https://en.wikipedia.org/wiki/Observer_pattern).

The `Notifiable` trait has the following methods:

```rust
fn add_subscriber(&mut self, subscriber: Weak<RefCell<dyn Notifiable>>)
```
> This method is used by a Notifiable implementation, the subscriber (i.e., observer), to subscribe to (i.e., observe) the Market events of the Notifiable implementation.

```rust
fn on_event(&mut self, event: Event)
```
> This method is used to notify the subscribers of a Market event.

We establish Market implementations also be Notifiable using Trait extensions: `pub trait Market : Notifiable`.

As consequences:

- A market implementation can subscribe to the Market events generated by another Market implementation using
  the `add_subscriber` method of the latter, passing its own reference as subscriber.
- A market implementation at which the trader buys or sells generates the corresponding Buy or Sell event and invokes
  the `on_event` method of all its subscribers to notify them.
- Although subscribers are ultimately implementation of the `Market` trait (which extends the `Notifiable` trait), they
  are passed as `dyn Notifiable`, making it impossible to erroneously invoke their `Market` trait methods (only the
  trader can do so).

#### the Event struct
in order to communicate to the other markets what has appended, a market use the `Event` struct, witch is a struct
with the following fields:
- `kind: EventKind`: an enum with the values: `Bought`,`Sold`,`LockedBuy`,`LockedSell`,`Wait`
- `good_kind: GoodKind`: the kind of the good (not the default one) is being bought/sold
- `quantity: f32`: the quantity of the good (not the default one) is being bought/sold
- `price: f32`: the quantity of the default good is being used to buy or sell

NOTE: this also have he Wait kind, if the trader wants to wait some time and let the market update,
he can call the macro:
```rust
wait_one_day!(market_1, market_2, market_3);
```
and all the markets will receive an event with kind: `Wait`

### Market Logs
each market has to provide a log file, and it should follow the following standard:

each market has to create a file log_market_name.txt, logs are in the following format:

<market_name>|YY:MM:DD:HH:SEC:MSES|<log_code>\n


where log codes are one of the following:

#### log for the buy
TRADER_LOCK_BUY-<trader_name>-GOOD_KIND:<good_kind>-EXCHANGE_QTY:<quantity>-LOCKED_QTY:<quantity>-TOKEN:<token>
TRADER_BUY-TOKEN:<token>  
MARKET_UNLOCK_BUY-TOKEN:<token>

#### log for the sell
TRADER_LOCK_SELL-<trader_name>-GOOD_KIND:<good_kind>-EXCHANGE_QTY:<quantity>-LOCKED_QTY:<quantity>-TOKEN:<token>
TRADER_SELL-TOKEN:<token>  
MARKET_UNLOCK_SELL-TOKEN:<token>

#### log for initialization

\nMARKET_INITIALIZATION\n\n

<good_kind>: good_quantity\n  
<good_kind>: good_quantity\n  
………………………………..  
<good_kind>: good_quantity\n  
END_MARKET_INITIALIZATION

all the quantities will use the :+e formatter (exponential)
all kinds will use the good.to_string() method


### Market prices fluctuation

#### how to determine prices
each market has internal sell and buy prices for each good, the buy price must always be slightly cheaper than the buy price,
the exact percentage is left to each team.

the baseline price has to be determined with a supply rule, for example... if a trader has €1000 and $2000 the exchange rate € to $ is 2,
which means that you can buy 2 dollars for one euro. (plus a certain percentage for buy, minus a certain percentage for the sell price)
this is the general logic, each team is free to slightly modify it to fit their needs and make their market more interesting.

#### how to vary quantities

now the prices will change with the change of the quantity, here is a set of guidelines for the quantity fluctuation

NOTE 0: the action of generating a good internally is referred to as "trade" from now on,
a callback to the update function of the observer will be referred to as "day"

- a market can't create a currency out of anything, he always has to give something in exchange
- the market can trade only one kind of currency every day.
- each individual trade must be smaller than €10'000 (calculated using the default exchange rate).
- for each good kind a market can decide to be an importer or an exporter.  
  take for example the USD, a market which uses EUR (or another good kind) to generate USD is an importer,
  a market that uses USD to generate EUR (or another good kind) is an exporter.
- a market can decide to switch between exporter and producer, but he can do this choice once every 100 days.
- Every time a market uses an internal trade there is a 5% possibility of a supply shortage, which means the used trade becomes unavailable for 100 days.
- the internal exchange rate the market uses must be in the +- 25% range of the default exchange rate.


NOTE 1: if you want to trade YUAN for YEN you will need to calculate the exchange rate, since it is not in the constants,
in this way: YUAN to YEN = (EUR to YEN) / (EUR to YUAN) = 144/7.4 = 19,4.

NOTE 2: these are guidelines, designed to create an environment where making money is not too easy and not too hard.
making small changes is not only not forbidden, but also encouraged since having the markets behave a little bit differently from one another is important.
you can implement any logic behind this and also change the default values.

for example:
- you can make it so that the chance of a supply shortage increases if a trade is overused.
- you can make that the maximum quantity you can generate every day is a periodic function.
- you can allow your market to change from importer to exporter immediately (instead of wearing 100 days) but this would cost €1000
  or many other this, as long as they are somewhat sensible

keep in mind that your team's final objective is to sell as many copies as you can at the market fair.
and to do that you have to create a market that is fun to interact with, from the trader's point of view.
making money while trading (or even making other weird strategies) should be a fun problem to solve, but should not be impossible.

## Code Testing
These specs also dictate how the testing of the common code is to be run. This paragraph tells which modules need to be tested.

It is the WG's duty to spell out which unit tests are to be carried out and the goal of each unit test function.
Moreover, you should arrange the workload of coding these tests amongst the groups.

there are two kinds of tests:
- tests for the common code
- tests for the market implementations

you can find a list of all the tests to implement and run [here](https://github.com/WG-AdvancedProgramming/market-protocol-specifications/blob/main/tests_to_do.md)

### tests for the common code
this kind of tests are design to make sure that the library we are creating works, and has to be written in the file: [market-common/src/tests.rs](https://github.com/WG-AdvancedProgramming/market-common/blob/main/src/tests.rs)

### tests for the market implementations
this kind of tests are design to make sure each implementation of a market works correctly, and has to be written in the file: [market-common/src/market/market_tests.rs](https://github.com/WG-AdvancedProgramming/market-common/blob/main/src/market/market_test.rs)
making this tests is a little bit complicated, since in the library we have only the traits, and not the implementations of market,
to know how to write, and run tests for the market read the aforementioned file.


## Code and Usage
All common code needs to be commented, possibly with reference to this document and indicate the author of the comment.

Ultimately, there needs to be a number of shared files in the common repo that everyone must be able to download and build upon:
- Good: containing the Good struct, its Impl and its tests
- GoodKind, containing the GoodKind Enum
- GoodError, containing the GoodError Enum
- MarketError, containing the MarketError Enum
- MarketsTest, containing the unit tests for anything Market-related
- one market file per group

