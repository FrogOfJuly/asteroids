use super::market::History;
use crate::ob::order::{Amount, OrderSide};

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct AgentId(u64);

impl AgentId {
    pub fn new(id: u64) -> Self {
        AgentId(id)
    }
}

pub trait Agent {
    fn setup(&mut self, id: AgentId);

    fn produce_orders(&mut self, history: &History) -> Vec<(OrderSide, Amount, i64)>;
}
