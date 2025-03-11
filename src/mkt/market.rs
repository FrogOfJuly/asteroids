use std::{cell::RefCell, collections::HashMap, fmt::Display};

use crate::ob::{
    amount::Amount,
    book::{OrderBook, Transaction},
    orders::{flat::Order, limit::LimitOrder},
};

use super::{
    account::Account,
    agent::{Agent, AgentId},
};

#[derive(Default, Debug)]
pub struct History {
    pub step: RefCell<u64>,
    pub transactions: Vec<Transaction>,
    pub rejected_orders: Vec<Order>,
    pub unfulfilled_orders: Vec<Order>,
}

impl History {
    pub fn market_price(&self) -> Option<Amount> {
        let sum: i64 = self
            .transactions
            .iter()
            .map(|tr| (tr.ask_gain + tr.bid_loss).as_int / 2)
            .sum();

        if self.transactions.is_empty() {
            None
        } else {
            Some(Amount {
                as_int: sum / (self.transactions.len() as i64),
            })
        }
    }

    pub fn inc_step(&self) {
        *self.step.borrow_mut() += 1;
    }
}

impl Display for History {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "fulfilled: {:?}\nrejected: {:?}\nunfulfilled: {:?}",
            self.transactions, self.rejected_orders, self.unfulfilled_orders
        )
    }
}

pub struct MarketInfo {
    pub name: String,
    // commodity : CommodityType,
}

pub struct Market {
    pub history: History,
    pub info: MarketInfo,

    book: OrderBook,

    id: RefCell<u64>,

    market_account: Account,

    accounts: HashMap<AgentId, Account>,
    agents: HashMap<AgentId, RefCell<Box<dyn Agent>>>,
    order_map: HashMap<u64, AgentId>,
}

impl Market {
    pub fn new(info: MarketInfo) -> Market {
        Self {
            book: Default::default(),
            history: Default::default(),
            info,
            id: Default::default(),
            market_account: Default::default(),
            accounts: Default::default(),
            agents: Default::default(),
            order_map: Default::default(),
        }
    }

    pub fn register_agent(&mut self, mut agent: Box<dyn Agent>) -> AgentId {
        let id = AgentId::new(*self.id.borrow());
        *self.id.borrow_mut() += 1;
        agent.setup(id, &self.info);
        self.accounts.insert(id, Account::starting_account());
        self.agents.insert(id, RefCell::new(agent));
        id
    }

    pub fn owner(&self, order: &LimitOrder) -> Option<AgentId> {
        let order_id = Into::<Order>::into(*order).id;
        self.order_map.get(&order_id).cloned()
    }

    pub fn account(&self, agent_id: AgentId) -> Option<Account> {
        self.accounts.get(&agent_id).cloned()
    }

    pub fn step(&mut self) -> usize {
        self.order_map.clear();
        self.clear_reservations();

        let rejected_orders = self.process_agent_actions();

        let market_price = self.history.market_price();
        self.history = History::default();

        self.history.rejected_orders = rejected_orders;

        let market_transactions = self.book.match_all_market(market_price);

        market_transactions
            .iter()
            .for_each(|trns| self.fulfill_transaction(trns));

        let limit_transactions = self.book.match_all_limit();

        limit_transactions
            .iter()
            .for_each(|trns| self.fulfill_transaction(trns));

        self.history.transactions = [market_transactions, limit_transactions].concat();
        self.history.unfulfilled_orders = self.book.all_orders();
        self.history.inc_step();

        self.history.transactions.len()
    }
}

impl Market {
    fn clear_reservations(&mut self) {
        self.accounts.iter_mut().for_each(|(_, acc_mut)| {
            acc_mut.reserved_commodity = Default::default();
            acc_mut.reserved_money = Default::default();
        });
    }

    fn process_agent_actions(&mut self) -> Vec<Order> {
        let mut rejected_orders: Vec<Order> = Vec::new();
        let mut agents: HashMap<AgentId, RefCell<Box<dyn Agent>>> = Default::default();

        std::mem::swap(&mut self.agents, &mut agents);

        agents.iter().for_each(|(&agent_id, agent_ref)| {
            let Some(account_copy) = self.account(agent_id) else {
                return;
            };

            let orders: Vec<_> = agent_ref
                .borrow_mut()
                .produce_orders(&account_copy, &self.info, &self.history)
                .into_iter()
                .flat_map(|data| self.book.new_order_checked(data))
                .collect();

            orders.into_iter().for_each(|order| {
                let reserved = self
                    .accounts
                    .get_mut(&agent_id)
                    .unwrap()
                    .reserve_order(order);

                if reserved {
                    let order_id = Into::<Order>::into(order).id;
                    self.order_map.insert(order_id, agent_id);
                    self.book.add_order(order);
                } else {
                    rejected_orders.push(order);
                }
            });
        });

        std::mem::swap(&mut self.agents, &mut agents);

        rejected_orders
    }

    fn fulfill_transaction(&mut self, trns: &Transaction) {
        let Some(bidder_id) = self.order_map.get(&trns.bid_id) else {
            panic!("{:?} has no bidder", trns);
        };
        let Some(asker_id) = self.order_map.get(&trns.ask_id) else {
            panic!("{:?} has no asker", trns);
        };

        let Some(bidder_acc) = self.accounts.get_mut(bidder_id) else {
            panic!("bidder of {:?} has no account", trns);
        };

        self.market_account.money += trns.diff;

        bidder_acc.commodity += trns.size;
        bidder_acc.money -= trns.bid_loss;

        let Some(asker_acc) = self.accounts.get_mut(asker_id) else {
            panic!("asker of {:?} has no account", trns);
        };

        asker_acc.commodity -= trns.size;
        asker_acc.money += trns.ask_gain;
    }
}
