use crate::{
    amount::Amount,
    orders::{
        flat::{Order, OrderData, OrderSide},
        limit::LimitOrder,
        market::MarketOrder,
    },
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Account {
    pub commodity: i64,
    pub money: Amount,
    pub reserved_money: Amount,
    pub reserved_commodity: i64,
    pub dept: Amount,
}

impl Account {
    pub fn starting_account() -> Self {
        Account {
            commodity: 10,
            money: Amount { as_int: 10 },
            ..Default::default()
        }
    }

    pub fn reservable(&self, orders: &[&OrderData]) -> bool {
        let (money, comm) = orders.iter().fold((0, 0), |(l, r), &data| match data.side {
            OrderSide::Bid => (l + data.price.unwrap_or_default().as_int * data.size, r),
            OrderSide::Ask => (l, r + data.size),
        });

        self.commodity >= self.reserved_commodity + comm
            && self.money >= self.reserved_money + Amount { as_int: money }
    }

    pub fn reserve_order(&mut self, order: Order) -> bool {
        if let Result::Ok(limit_order) = order.try_into() {
            self.reserve_limit_order(&limit_order)
        } else if let Result::Ok(market_order) = order.try_into() {
            self.reserve_market_order(&market_order)
        } else {
            false
        }
    }

    fn reserve_market_order(&mut self, order: &MarketOrder) -> bool {
        match order {
            MarketOrder::BidOrder { .. } => true,
            MarketOrder::AskOrder { data } => {
                let maximum_commodity_transfer = data.size;

                if maximum_commodity_transfer + self.reserved_commodity <= self.commodity {
                    self.reserved_commodity += maximum_commodity_transfer;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn reserve_limit_order(&mut self, order: &LimitOrder) -> bool {
        match order {
            LimitOrder::BidOrder { data } => {
                let maximum_transaction = data.price * data.size;

                if maximum_transaction.as_int + self.reserved_money.as_int <= self.money.as_int {
                    self.reserved_money += maximum_transaction;
                    true
                } else {
                    false
                }
            }
            LimitOrder::AskOrder { data } => {
                let maximum_commodity_transfer = data.size;

                if maximum_commodity_transfer + self.reserved_commodity <= self.commodity {
                    self.reserved_commodity += maximum_commodity_transfer;
                    true
                } else {
                    false
                }
            }
        }
    }
}
