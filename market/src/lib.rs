#![feature(associated_type_defaults)]
#![feature(inherent_associated_types)]
#![feature(trait_alias)]

pub mod account;
pub mod agent;
pub mod amount;
pub mod book;
pub mod market;
pub mod orders;

#[cfg(test)]
mod simulation_test {
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
            let mut agents: Vec<Market<CommodityType>::AgentRefType> = vec![];

            agents.push(RefCell::new(Box::new(ConsumerAgent {})));
            agents.push(RefCell::new(Box::new(ProducerAgent {})));

            agents
                .into_iter()
                .map(|agent| (market.register_with_default_acc(), agent))
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

            market.clear_orders();
        }

        println!("history: {}", history);
    }
}
