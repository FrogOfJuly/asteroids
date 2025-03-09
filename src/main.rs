use mkt::market::*;

pub mod mkt;
pub mod ob;

fn main() {
    let mut market = Market::default();
    market.step();
}
