use market::{
    agent::Agent,
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

pub struct BuyAgent<T> {
    pub ask_size: i64,
    pub ask_amount: i64,
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
                size: self.ask_size,
            };
            self.ask_amount as usize
        ]
    }
}

pub struct IncBuyAgent<T> {
    pub ask_size: i64,
    pub ask_amount: i64,
    pub period: u64,
    pub price: Amount,
    pub _ph: std::marker::PhantomData<T>,
}

impl<T> Agent for IncBuyAgent<T> {
    type CommodityType = T;

    fn setup(&mut self, _id: market::agent::AgentId, _info: &Self::MarketInfoType) {
        todo!()
    }

    fn produce_orders(
        &mut self,
        _account: &market::account::Account,
        _info: &Self::MarketInfoType,
        _history: &market::market::History,
    ) -> Vec<OrderData> {
        // permit checks about your orders
        todo!()
    }
}
