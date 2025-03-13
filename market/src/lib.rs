#![allow(incomplete_features)]
#![feature(associated_type_defaults)]
#![feature(inherent_associated_types)]
#![feature(trait_alias)]

pub mod account;
pub mod agent;
pub mod amount;
pub mod market;
pub mod order_book;
pub mod orders;

#[cfg(test)]
mod test_simulation {
    use std::cell::RefCell;

    use crate::{
        account::Account,
        agent::{Agent, AgentId},
        market::{History, Market, MarketInfo},
        orders::flat::OrderData,
    };

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

    #[test]
    fn run() {
        let mut market = Market::new(MarketInfo {
            name: "test".to_owned(),
            commodity: CommodityType::Unit,
        });
        let agents: Vec<_> = {
            let a1: Market<CommodityType>::AgentRefType = RefCell::new(Box::new(ConsumerAgent {}));
            let a2: Market<CommodityType>::AgentRefType = RefCell::new(Box::new(ProducerAgent {}));

            [a1, a2]
                .into_iter()
                .map(|agent| {
                    let id = market.register_with_default_acc();
                    (*agent.borrow_mut()).setup(id, &market.info);
                    (id, agent)
                })
                .collect()
        };

        let mut history = History::default();

        for step in 1..=10 {
            println!("history: {}", history);

            let rejected_orders = market.agents_submit_orders(agents.as_slice(), &history);
            let transactions = market.process_submitted_orders(history.market_price());
            let unfulfilled_orders = market.all_orders();

            history = History {
                step,
                transactions,
                rejected_orders,
                unfulfilled_orders,
            };

            market.clear_reserves_and_orders();
        }

        println!("history: {}", history);
    }
}
