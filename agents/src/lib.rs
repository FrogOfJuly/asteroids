use std::vec;

use market::{
    agent::{Agent, AgentId},
    amount::Amount,
    orders::flat::{OrderData, OrderSide},
};

pub struct IdleAgent<T> {
    _ph: std::marker::PhantomData<T>,
}

impl<T> Agent for IdleAgent<T> {
    type CommodityType = T;

    fn setup(&mut self, _id: market::agent::AgentId, _info: &Self::MarketInfoType) {}

    fn produce_orders(
        &mut self,
        _account: &market::account::Account,
        _info: &Self::MarketInfoType,
        _history: &market::market::History,
    ) -> Vec<market::orders::flat::OrderData> {
        vec![]
    }
}

pub struct SellAgent<T> {
    pub ask_size: i64,
    pub ask_amount: i64,
    pub period: u64,
    pub innate_price: Option<Amount>,
    pub _ph: std::marker::PhantomData<T>,
}

impl<T> Agent for SellAgent<T> {
    type CommodityType = T;

    fn setup(&mut self, _id: market::agent::AgentId, _info: &Self::MarketInfoType) {}

    fn produce_orders(
        &mut self,
        account: &market::account::Account,
        _info: &Self::MarketInfoType,
        history: &market::market::History,
    ) -> Vec<market::orders::flat::OrderData> {
        if account.commodity == 0 || history.step % self.period != 0 {
            return vec![];
        }

        vec![
            OrderData {
                side: OrderSide::Ask,
                price: history.market_price().or(self.innate_price),
                size: account.commodity.min(self.ask_size),
            };
            self.ask_amount as usize
        ]
    }
}

#[derive(Clone, Debug)]
pub struct BuyAgent<T> {
    pub bid_size: i64,
    pub bid_amount: i64,
    pub period: u64,
    pub innate_price: Option<Amount>,
    pub _ph: std::marker::PhantomData<T>,
}

impl<T> Agent for BuyAgent<T> {
    type CommodityType = T;

    fn setup(&mut self, _id: market::agent::AgentId, _info: &Self::MarketInfoType) {}

    fn produce_orders(
        &mut self,
        account: &market::account::Account,
        _info: &Self::MarketInfoType,
        history: &market::market::History,
    ) -> Vec<market::orders::flat::OrderData> {
        if account.money.as_int == 0 || history.step % self.period != 0 {
            return vec![];
        }

        vec![
            OrderData {
                side: OrderSide::Bid,
                price: history.market_price().or(self.innate_price),
                size: self.bid_size,
            };
            self.bid_amount as usize
        ]
    }
}

pub struct IncBuyAgent<T> {
    pub bid_size: i64,
    pub bid_amount: i64,
    pub period: u64,

    pub my_id: AgentId,

    pub price: Amount,
    pub increment: Amount,

    pub _ph: std::marker::PhantomData<T>,
}

impl<T> Agent for IncBuyAgent<T> {
    type CommodityType = T;

    fn setup(&mut self, id: market::agent::AgentId, _info: &Self::MarketInfoType) {
        self.my_id = id;
    }

    fn produce_orders(
        &mut self,
        account: &market::account::Account,
        _info: &Self::MarketInfoType,
        history: &market::market::History,
    ) -> Vec<OrderData> {
        if history.step % self.period != 0 {
            return vec![];
        }

        if account.money.as_int == 0 {
            return vec![];
        }

        let missing_comm: i64 = history
            .filter_by_agent_id(&self.my_id)
            .unfulfilled_orders
            .iter()
            .map(|(_, o)| o.size)
            .sum();

        if missing_comm > 0 {
            self.price = Amount {
                as_int: (self.price.as_int - self.increment.as_int).max(0),
            };
        } else {
            self.price += self.increment;
        }

        if self.price.as_int <= 0 {
            return vec![
                OrderData {
                    side: OrderSide::Bid,
                    price: None,
                    size: self.bid_size,
                };
                self.bid_amount as usize
            ];
        }

        let units = account.money.as_int / self.price.as_int;

        if units <= 0 {
            return vec![];
        };

        let order_num = (units / self.bid_size).min(self.bid_amount);

        if order_num <= 0 {
            return vec![];
        }

        vec![
            OrderData {
                side: OrderSide::Bid,
                price: Some(self.price),
                size: self.bid_size,
            };
            order_num as usize
        ]
    }
}

pub struct IncSellAgent<T> {
    pub ask_size: i64,
    pub ask_amount: i64,
    pub period: u64,

    pub my_id: AgentId,

    pub price: Amount,
    pub increment: Amount,

    pub _ph: std::marker::PhantomData<T>,
}

impl<T> Agent for IncSellAgent<T> {
    type CommodityType = T;

    fn setup(&mut self, id: market::agent::AgentId, _info: &Self::MarketInfoType) {
        self.my_id = id;
    }

    fn produce_orders(
        &mut self,
        account: &market::account::Account,
        _info: &Self::MarketInfoType,
        history: &market::market::History,
    ) -> Vec<OrderData> {
        if history.step % self.period != 0 {
            return vec![];
        }

        if account.commodity == 0 {
            return vec![];
        }

        let missing_comm: i64 = history
            .filter_by_agent_id(&self.my_id)
            .unfulfilled_orders
            .iter()
            .map(|(_, o)| o.size)
            .sum();

        if missing_comm > 0 {
            self.price = Amount {
                as_int: (self.price.as_int - self.increment.as_int).max(0),
            };
        } else {
            self.price += self.increment;
        }

        if self.price.as_int <= 0 {
            return vec![
                OrderData {
                    side: OrderSide::Ask,
                    price: None,
                    size: self.ask_size,
                };
                self.ask_amount as usize
            ];
        }

        let order_num = (account.commodity / self.ask_size).min(self.ask_amount);

        if order_num <= 0 {
            return vec![];
        }

        vec![
            OrderData {
                side: OrderSide::Bid,
                price: Some(self.price),
                size: self.ask_size,
            };
            order_num as usize
        ]
    }
}
