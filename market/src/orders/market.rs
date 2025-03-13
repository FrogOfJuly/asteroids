use crate::orders::flat::OrderSide;

use super::flat::{MarketOrderData, Order};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MarketOrder {
    BidOrder { data: MarketOrderData },
    AskOrder { data: MarketOrderData },
}

impl TryFrom<Order> for MarketOrder {
    type Error = ();

    fn try_from(value: Order) -> Result<Self, Self::Error> {
        use MarketOrder::*;
        use OrderSide::*;
        let data: MarketOrderData = value.try_into()?;
        match value.side {
            Bid => Result::Ok(BidOrder { data }),
            Ask => Result::Ok(AskOrder { data }),
        }
    }
}

impl From<MarketOrder> for Order {
    fn from(value: MarketOrder) -> Self {
        use MarketOrder::*;
        use OrderSide::*;
        match value {
            BidOrder { data } => Self {
                timestamp: data.timestamp,
                id: data.id,
                price: None,
                size: data.size,
                side: Bid,
            },
            AskOrder { data } => Self {
                timestamp: data.timestamp,
                id: data.id,
                price: None,
                size: data.size,
                side: Ask,
            },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct BidMarketOrder {
    pub data: MarketOrderData,
}

impl From<BidMarketOrder> for MarketOrderData {
    fn from(value: BidMarketOrder) -> Self {
        value.data
    }
}

impl From<MarketOrderData> for BidMarketOrder {
    fn from(value: MarketOrderData) -> Self {
        Self { data: value }
    }
}

impl From<BidMarketOrder> for MarketOrder {
    fn from(value: BidMarketOrder) -> Self {
        MarketOrder::BidOrder { data: value.data }
    }
}

impl From<BidMarketOrder> for Order {
    fn from(value: BidMarketOrder) -> Self {
        MarketOrder::BidOrder { data: value.data }.into()
    }
}

impl PartialOrd for BidMarketOrder {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BidMarketOrder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let timestamp_order = self.data.timestamp.cmp(&other.data.timestamp).reverse();
        match timestamp_order {
            std::cmp::Ordering::Equal => self.data.id.cmp(&other.data.id).reverse(),
            _ => timestamp_order,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct AskMarketOrder {
    pub data: MarketOrderData,
}

impl From<AskMarketOrder> for MarketOrderData {
    fn from(value: AskMarketOrder) -> Self {
        value.data
    }
}

impl From<MarketOrderData> for AskMarketOrder {
    fn from(value: MarketOrderData) -> Self {
        Self { data: value }
    }
}

impl From<AskMarketOrder> for MarketOrder {
    fn from(value: AskMarketOrder) -> Self {
        MarketOrder::AskOrder { data: value.data }
    }
}

impl From<AskMarketOrder> for Order {
    fn from(value: AskMarketOrder) -> Self {
        MarketOrder::AskOrder { data: value.data }.into()
    }
}

impl PartialOrd for AskMarketOrder {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AskMarketOrder {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        let timestamp_order = self.data.timestamp.cmp(&other.data.timestamp).reverse();
        match timestamp_order {
            std::cmp::Ordering::Equal => self.data.id.cmp(&other.data.id).reverse(),
            _ => timestamp_order,
        }
    }
}
