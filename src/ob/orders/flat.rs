use crate::ob::amount::Amount;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum OrderSide {
    Bid, // buy
    Ask, // sell
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct FlatOrder {
    pub timestamp: i64,
    pub id: u64,
    pub side: OrderSide,
    pub price: Option<Amount>,
    pub size: i64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct LimitOrderData {
    pub timestamp: i64,
    pub id: u64,
    pub price: Amount,
    pub size: i64,
}

impl TryFrom<FlatOrder> for LimitOrderData {
    type Error = ();

    fn try_from(value: FlatOrder) -> Result<Self, Self::Error> {
        let Some(price) = value.price else {
            return Result::Err(());
        };

        Result::Ok(Self {
            timestamp: value.timestamp,
            id: value.id,
            price,
            size: value.size,
        })
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MarketOrderData {
    pub timestamp: i64,
    pub id: u64,
    pub size: i64,
}

impl TryFrom<FlatOrder> for MarketOrderData {
    type Error = ();

    fn try_from(value: FlatOrder) -> Result<Self, Self::Error> {
        let None = value.price else {
            return Result::Err(());
        };

        Result::Ok(Self {
            timestamp: value.timestamp,
            id: value.id,
            size: value.size,
        })
    }
}
