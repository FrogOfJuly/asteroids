use std::cell::RefCell;

use agents::{BuyAgent, IncBuyAgent, IncSellAgent, SellAgent};
use market::{
    account::Account,
    agent::{Agent as GenericAgent, AgentId},
    amount::Amount,
    market::{History, Market, MarketInfo},
};

pub enum CommodityType {
    Unit,
}

trait Agent =
    GenericAgent<CommodityType = CommodityType, MarketInfoType = MarketInfo<CommodityType>>;

type AgentRefType = RefCell<Box<dyn Agent>>;

pub struct MarketConfiguration {
    pub market: Market<CommodityType>,
    pub agents: Vec<(AgentId, AgentRefType)>,

    pub history: History,
    pub market_price: Option<Amount>,
    pub step: u64,
}

impl MarketConfiguration {
    pub fn new() -> Self {
        let mut market = Market::new(MarketInfo {
            name: "test".to_owned(),
            commodity: CommodityType::Unit,
        });

        let simpl_buy_agents: Vec<Box<dyn Agent>> = std::iter::repeat(())
            .map(|_| BuyAgent::<CommodityType> {
                bid_size: 1,
                bid_amount: 1,
                period: 1,
                innate_price: Some(Amount { as_int: 50 }),
                _ph: std::marker::PhantomData,
            })
            .map(Box::new)
            .take(3)
            .map(|x| {
                let y: Box<dyn Agent> = x;
                y
            })
            .collect();

        let simpl_sell_agents: Vec<Box<dyn Agent>> = std::iter::repeat(())
            .map(|_| SellAgent::<CommodityType> {
                ask_size: 1,
                ask_amount: 2,
                period: 2,
                innate_price: Some(Amount { as_int: 25 }),
                _ph: std::marker::PhantomData,
            })
            .map(Box::new)
            .take(3)
            .map(|x| {
                let y: Box<dyn Agent> = x;
                y
            })
            .collect();

        let inc_sell_agents: Vec<Box<dyn Agent>> = std::iter::repeat(())
            .map(|_| IncSellAgent::<CommodityType> {
                ask_size: 1,
                ask_amount: 2,
                period: 1,
                my_id: AgentId::new(0),
                price: Amount::new(),
                increment: Amount { as_int: 1 },
                _ph: std::marker::PhantomData,
            })
            .map(Box::new)
            .take(10)
            .map(|x| {
                let y: Box<dyn Agent> = x;
                y
            })
            .collect();

        let inc_buy_agents: Vec<Box<dyn Agent>> = std::iter::repeat(())
            .map(|_| IncBuyAgent::<CommodityType> {
                bid_size: 1,
                bid_amount: 2,
                period: 1,
                my_id: AgentId::new(0),
                price: Amount::new(),
                increment: Amount { as_int: 1 },
                _ph: std::marker::PhantomData,
            })
            .map(Box::new)
            .take(10)
            .map(|x| {
                let y: Box<dyn Agent> = x;
                y
            })
            .collect();

        let agents: Vec<_> = [
            simpl_buy_agents,
            simpl_sell_agents,
            inc_buy_agents,
            inc_sell_agents,
        ]
        .into_iter()
        .flat_map(|x| x.into_iter())
        .map(|mut agent| {
            let id = market.register_with_acc(Account {
                commodity: 100,
                money: Amount { as_int: 1000 },
                ..Default::default()
            });
            agent.setup(id, &market.info);
            (id, RefCell::new(agent))
        })
        .collect();

        MarketConfiguration {
            market,
            agents,
            history: Default::default(),
            market_price: Default::default(),
            step: 1,
        }
    }

    pub fn step(&mut self) -> Option<Amount> {
        let rejected_orders = self
            .market
            .agents_submit_orders(self.agents.as_slice(), &self.history);
        let transactions = self.market.process_submitted_orders(self.market_price);
        let unfulfilled_orders = self.market.all_orders();

        self.history = History {
            step: self.step,
            transactions,
            rejected_orders,
            unfulfilled_orders,
        };

        if !self.history.no_transactions() {
            self.market_price = self.history.market_price()
        }

        self.market.clear_reserves_and_orders();
        self.step += 1;
        self.market_price
    }
}

impl Default for MarketConfiguration {
    fn default() -> Self {
        Self::new()
    }
}
