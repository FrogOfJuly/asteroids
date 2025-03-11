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
    type CommodityType;
    type MarketInfoType = MarketInfo<Self::CommodityType>;

    fn setup(&mut self, id: AgentId, info: &Self::MarketInfoType);

    fn produce_orders(
        &mut self,
        account: &Account,
        info: &Self::MarketInfoType,
        history: &History,
    ) -> Vec<OrderData>;
}
