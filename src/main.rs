use mkt::{agent::*, market::*};
use ob::orders::flat::OrderData;

pub mod mkt;
pub mod ob;

struct ProducerAgent {}

impl Agent for ProducerAgent {
    fn setup(&mut self, _id: AgentId, _info: &MarketInfo) {}

    fn produce_orders(
        &mut self,
        _account: &Account,
        _info: &MarketInfo,
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
    fn setup(&mut self, _id: AgentId, _info: &MarketInfo) {}

    fn produce_orders(
        &mut self,
        _account: &Account,
        _info: &MarketInfo,
        _history: &History,
    ) -> Vec<OrderData> {
        vec!["B:2:1".try_into().unwrap(), "B:4:1".try_into().unwrap()]
    }
}

fn main() {
    let mut market = Market::new(MarketInfo {
        name: "test".to_owned(),
    });
    market.register_agent(Box::new(ConsumerAgent {}));
    market.register_agent(Box::new(ProducerAgent {}));

    while market.step() > 0 {
        println!("{:?}", market.history.market_price());
    }
}
