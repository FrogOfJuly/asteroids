use super::{
    account::Account,
    market::{History, MarketInfo},
};
use crate::ob::orders::flat::OrderData;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct AgentId(u64);

impl AgentId {
    pub fn new(id: u64) -> Self {
        AgentId(id)
    }
}

pub trait Agent {
    fn setup(&mut self, id: AgentId, info: &MarketInfo);

    fn produce_orders(
        &mut self,
        account: &Account,
        info: &MarketInfo,
        history: &History,
    ) -> Vec<OrderData>;
}
