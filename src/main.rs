use mkt::{
    agent::{Agent, AgentId},
    market::*,
};
use ob::{amount::Amount, orders::flat::OrderSide};

pub mod mkt;
pub mod ob;

struct ProducerAgent {
    id: AgentId,
}

impl Agent for ProducerAgent {
    fn setup(&mut self, id: mkt::agent::AgentId) {
        self.id = id;
    }

    fn produce_orders(&mut self, _history: &History) -> Vec<(OrderSide, Amount, i64)> {
        vec![
            (OrderSide::Ask, Amount { as_int: 1 }, 1),
            (OrderSide::Ask, Amount { as_int: 2 }, 1),
            (OrderSide::Ask, Amount { as_int: 3 }, 1),
        ]
    }
}

struct ConsumerAgent {
    id: AgentId,
}

impl Agent for ConsumerAgent {
    fn setup(&mut self, id: AgentId) {
        self.id = id;
    }

    fn produce_orders(&mut self, _history: &History) -> Vec<(OrderSide, Amount, i64)> {
        vec![
            (OrderSide::Bid, Amount { as_int: 2 }, 1),
            (OrderSide::Bid, Amount { as_int: 4 }, 1),
        ]
    }
}

fn main() {
    let mut market = Market::default();
    market.register_agent(Box::new(ConsumerAgent {
        id: AgentId::new(0),
    }));
    market.register_agent(Box::new(ProducerAgent {
        id: AgentId::new(0),
    }));

    market.step();

    println!("{}", market.history)
}
