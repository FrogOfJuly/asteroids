#![feature(associated_type_defaults)]

use mkt::{account::Account, agent::*, market::*};
use ob::orders::flat::OrderData;

pub mod mkt;
pub mod ob;

enum CommodityType {
    Unit,
}

struct ProducerAgent {}

impl Agent for ProducerAgent {
    type CommodityType = CommodityType;

    fn setup(&mut self, _id: AgentId, _info: &MarketInfo<CommodityType>) {}

    fn produce_orders(
        &mut self,
        _account: &Account,
        _info: &MarketInfo<CommodityType>,
        _history: &History,
    ) -> Vec<OrderData> {
        vec![
            "A:1:1".try_into().unwrap(),
            "A:2:1".try_into().unwrap(),
            "A:3:1".try_into().unwrap(),
        ]
    }
}

struct ConsumerAgent {}

impl Agent for ConsumerAgent {
    type CommodityType = CommodityType;

    fn setup(&mut self, _id: AgentId, _info: &Self::MarketInfoType) {}

    fn produce_orders(
        &mut self,
        _account: &Account,
        _info: &MarketInfo<CommodityType>,
        _history: &History,
    ) -> Vec<OrderData> {
        vec!["B:2:1".try_into().unwrap(), "B:4:1".try_into().unwrap()]
    }
}

fn main() {
    let mut market = Market::new(MarketInfo {
        name: "test".to_owned(),
        commodity: CommodityType::Unit,
    });
    market.register_agent(Box::new(ConsumerAgent {}));
    market.register_agent(Box::new(ProducerAgent {}));

    while market.step() > 0 {
        println!("{:?}", market.history.market_price());
        market.history.inc_step();
    }
}
