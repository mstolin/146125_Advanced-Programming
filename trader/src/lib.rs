use std::cell::RefCell;
use std::rc::Rc;
use unitn_market_2022::market::Market;

pub mod strategy;
mod tests;
pub mod trader;
mod consts;

type MarketRef = Rc<RefCell<dyn Market>>;
