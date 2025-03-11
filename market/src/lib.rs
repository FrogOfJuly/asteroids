#![feature(associated_type_defaults)]

pub mod mkt;
pub mod ob;

#[cfg(test)]
mod simulation_test {
    use crate::mkt::{account::Account, agent::*, market::*};
    use crate::ob::orders::flat::OrderData;
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
        market.register_with_starting_acc(Box::new(ConsumerAgent {}));
        market.register_with_starting_acc(Box::new(ProducerAgent {}));

        while market.step() > 0 {
            println!("{:?}", market.history);
            market.history.inc_step();
        }

        println!("{}", market.history);
    }
}
