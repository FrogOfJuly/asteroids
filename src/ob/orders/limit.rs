use crate::ob::orders::flat::OrderSide;

use super::flat::{FlatOrder, LimitOrderData};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LimitOrder {
    BidOrder { data: LimitOrderData },
    AskOrder { data: LimitOrderData },
}

impl TryFrom<FlatOrder> for LimitOrder {
    type Error = ();

    fn try_from(value: FlatOrder) -> Result<Self, Self::Error> {
        use LimitOrder::*;
        use OrderSide::*;
        let data: LimitOrderData = value.try_into()?;
        match value.side {
            Bid => Result::Ok(BidOrder { data }),
            Ask => Result::Ok(AskOrder { data }),
        }
    }
}

impl From<LimitOrder> for FlatOrder {
    fn from(value: LimitOrder) -> Self {
        use LimitOrder::*;
        use OrderSide::*;
        match value {
            BidOrder { data } => Self {
                timestamp: data.timestamp,
                id: data.id,
                price: Some(data.price),
                size: data.size,
                side: Bid,
            },
            AskOrder { data } => Self {
                timestamp: data.timestamp,
                id: data.id,
                price: Some(data.price),
                size: data.size,
                side: Ask,
            },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BidLimitOrder {
    pub data: LimitOrderData,
}

impl From<BidLimitOrder> for LimitOrderData {
    fn from(value: BidLimitOrder) -> Self {
        value.data
    }
}

impl From<LimitOrderData> for BidLimitOrder {
    fn from(value: LimitOrderData) -> Self {
        Self { data: value }
    }
}

impl From<BidLimitOrder> for LimitOrder {
    fn from(value: BidLimitOrder) -> Self {
        LimitOrder::BidOrder { data: value.data }
    }
}

impl From<BidLimitOrder> for FlatOrder {
    fn from(value: BidLimitOrder) -> Self {
        LimitOrder::BidOrder { data: value.data }.into()
    }
}

impl PartialOrd for BidLimitOrder {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BidLimitOrder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // best bid orders are those that buy by higher price
        // provided equal price, the best order is the oldest one
        match (
            self.data.price.cmp(&other.data.price),
            self.data.timestamp.cmp(&other.data.timestamp),
            self.data.id.cmp(&other.data.id),
        ) {
            (std::cmp::Ordering::Equal, std::cmp::Ordering::Equal, c) => c.reverse(),
            (std::cmp::Ordering::Equal, c, _) => c.reverse(),
            (c, _, _) => c,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AskLimitOrder {
    pub data: LimitOrderData,
}

impl From<AskLimitOrder> for LimitOrderData {
    fn from(value: AskLimitOrder) -> Self {
        value.data
    }
}

impl From<LimitOrderData> for AskLimitOrder {
    fn from(value: LimitOrderData) -> Self {
        Self { data: value }
    }
}

impl From<AskLimitOrder> for LimitOrder {
    fn from(value: AskLimitOrder) -> Self {
        LimitOrder::AskOrder { data: value.data }
    }
}

impl From<AskLimitOrder> for FlatOrder {
    fn from(value: AskLimitOrder) -> Self {
        LimitOrder::AskOrder { data: value.data }.into()
    }
}

impl PartialOrd for AskLimitOrder {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AskLimitOrder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // best ask orders are those that sells by lowest price
        // provided equal price, the best order is the oldest one
        match (
            self.data.price.cmp(&other.data.price),
            self.data.timestamp.cmp(&other.data.timestamp),
            self.data.id.cmp(&other.data.id),
        ) {
            (std::cmp::Ordering::Equal, std::cmp::Ordering::Equal, c) => c.reverse(),
            (std::cmp::Ordering::Equal, c, _) => c.reverse(),
            (c, _, _) => c.reverse(),
        }
    }
}
