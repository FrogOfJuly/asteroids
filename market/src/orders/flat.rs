use crate::amount::Amount;

#[derive(Copy, Clone, Debug, Eq, Hash, PartialEq)]
pub enum OrderSide {
    Bid, // buy
    Ask, // sell
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Order {
    pub timestamp: i64,
    pub id: u64,
    pub side: OrderSide,
    pub price: Option<Amount>,
    pub size: i64,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct OrderData {
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

impl TryFrom<Order> for LimitOrderData {
    type Error = ();

    fn try_from(value: Order) -> Result<Self, Self::Error> {
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

impl TryFrom<Order> for MarketOrderData {
    type Error = ();

    fn try_from(value: Order) -> Result<Self, Self::Error> {
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

impl From<Order> for OrderData {
    fn from(
        Order {
            side, price, size, ..
        }: Order,
    ) -> Self {
        Self { side, price, size }
    }
}

impl TryFrom<&str> for OrderData {
    type Error = ();

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        value.to_owned().try_into()
    }
}

impl TryFrom<String> for OrderData {
    type Error = ();

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.trim().split(":").collect();

        let (side, price, size) = if let [side, price, size] = parts.as_slice() {
            Ok((side, Some(price), size))
        } else if let [side, size] = parts.as_slice() {
            Ok((side, None, size))
        } else {
            Err(())
        }?;

        let side = if *side == "A" {
            Ok(OrderSide::Ask)
        } else if *side == "B" {
            Ok(OrderSide::Bid)
        } else {
            return Err(());
        }?;

        let price = price
            .into_iter()
            .flat_map(|price| price.parse::<u64>().map_err(|_| ()))
            .map(|x| Amount { as_int: x as i64 })
            .next();

        let size: u64 = size.parse().map_err(|_| ())?;

        Ok(OrderData {
            side,
            price,
            size: size as i64,
        })
    }
}
