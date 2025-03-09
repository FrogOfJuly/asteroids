use std::ops::{AddAssign, Mul, MulAssign, Not, SubAssign};

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum OrderSide {
    Bid, // buy
    Ask, // sell
}

impl Not for OrderSide {
    type Output = OrderSide;

    fn not(self) -> OrderSide {
        match self {
            OrderSide::Bid => OrderSide::Ask,
            OrderSide::Ask => OrderSide::Bid,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Amount {
    pub as_int: i64, // "cents"
}

impl Amount {
    pub fn new() -> Self {
        Amount { as_int: 0 }
    }
}

impl Default for Amount {
    fn default() -> Self {
        Self::new()
    }
}

impl AddAssign for Amount {
    fn add_assign(&mut self, rhs: Self) {
        self.as_int += rhs.as_int;
    }
}

impl SubAssign for Amount {
    fn sub_assign(&mut self, rhs: Self) {
        self.as_int -= rhs.as_int;
    }
}

impl MulAssign<i64> for Amount {
    fn mul_assign(&mut self, rhs: i64) {
        self.as_int *= rhs;
    }
}

impl Mul<i64> for Amount {
    type Output = Self;
    fn mul(self, rhs: i64) -> Self {
        Amount {
            as_int: self.as_int * rhs,
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FlatLimitOrder {
    pub timestamp: i64,
    pub id: u64,
    pub side: OrderSide,
    pub price: Amount,
    pub size: i64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LimitOrderData {
    pub timestamp: i64,
    pub id: u64,
    pub price: Amount,
    pub size: i64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum LimitOrder {
    BidOrder { data: LimitOrderData },
    AskOrder { data: LimitOrderData },
}

impl From<FlatLimitOrder> for LimitOrderData {
    fn from(value: FlatLimitOrder) -> Self {
        Self {
            timestamp: value.timestamp,
            id: value.id,
            price: value.price,
            size: value.size,
        }
    }
}

impl From<FlatLimitOrder> for LimitOrder {
    fn from(value: FlatLimitOrder) -> Self {
        use LimitOrder::*;
        use OrderSide::*;
        match value.side {
            Bid => BidOrder { data: value.into() },
            Ask => AskOrder { data: value.into() },
        }
    }
}

impl From<LimitOrder> for FlatLimitOrder {
    fn from(value: LimitOrder) -> Self {
        use LimitOrder::*;
        use OrderSide::*;
        match value {
            BidOrder { data } => Self {
                timestamp: data.timestamp,
                id: data.id,
                price: data.price,
                size: data.size,
                side: Bid,
            },
            AskOrder { data } => Self {
                timestamp: data.timestamp,
                id: data.id,
                price: data.price,
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

impl From<BidLimitOrder> for FlatLimitOrder {
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
        ) {
            (std::cmp::Ordering::Equal, c) => c,
            (c, _) => c,
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

impl From<AskLimitOrder> for FlatLimitOrder {
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
        ) {
            (std::cmp::Ordering::Equal, c) => c,
            (c, _) => c.reverse(),
        }
    }
}
