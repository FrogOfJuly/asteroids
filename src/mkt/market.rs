use std::{cell::RefCell, collections::HashMap, fmt::Display};

use crate::ob::{
    amount::Amount,
    book::{OrderBook, Transaction},
    orders::{flat::Order, limit::LimitOrder, market::MarketOrder},
};

use super::agent::{Agent, AgentId};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub struct Account {
    pub commodity: i64,
    pub money: Amount,
    pub reserved_money: Amount,
    pub reserved_commodity: i64,
    pub dept: Amount,
}

impl Account {
    fn starting_account() -> Self {
        Account {
            commodity: 10,
            money: Amount { as_int: 10 },
            ..Default::default()
        }
    }
}

#[derive(Default, Debug)]
pub struct History {
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

#[derive(Default)]
pub struct Market {
    pub book: OrderBook,
    pub history: History,

    id: RefCell<u64>,

    accounts: HashMap<AgentId, Account>,
    agents: HashMap<AgentId, RefCell<Box<dyn Agent>>>,
    order_map: HashMap<u64, AgentId>,
}

impl Market {
    pub fn register_agent(&mut self, mut agent: Box<dyn Agent>) -> AgentId {
        let id = AgentId::new(*self.id.borrow());
        *self.id.borrow_mut() += 1;
        agent.setup(id);
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

    pub fn step(&mut self) {
        self.order_map.clear();
        self.clear_reserves();

        let rejected_orders = self.process_agent_actions();

        let market_price = self.history.market_price();
        self.history = History::default();

        self.history.rejected_orders = rejected_orders;

        let market_transactions = self.book.match_all_market(market_price);

        market_transactions
            .iter()
            .for_each(|trns| self.satisfy_transaction(trns));

        let limit_transactions = self.book.match_all_limit();

        limit_transactions
            .iter()
            .for_each(|trns| self.satisfy_transaction(trns));

        self.history.transactions = [market_transactions, limit_transactions].concat();
        self.history.unfulfilled_orders = self.book.all_orders();
    }
}

impl Market {
    fn clear_reserves(&mut self) {
        self.accounts.iter_mut().for_each(|(_, acc_mut)| {
            acc_mut.reserved_commodity = Default::default();
            acc_mut.reserved_money = Default::default();
        });
    }

    fn reserve_market(&mut self, agent_id: AgentId, order: &MarketOrder) -> bool {
        let Some(account_data) = self.account(agent_id) else {
            println!("Can't find account for {:?}", agent_id);
            return false;
        };

        match order {
            MarketOrder::BidOrder { .. } => true,
            MarketOrder::AskOrder { data } => {
                let maximum_commodity_transfer = data.size;

                if maximum_commodity_transfer + account_data.reserved_commodity
                    <= account_data.commodity
                {
                    self.accounts.get_mut(&agent_id).unwrap().reserved_commodity +=
                        maximum_commodity_transfer;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn reserve_limit(&mut self, agent_id: AgentId, order: &LimitOrder) -> bool {
        let Some(account_data) = self.account(agent_id) else {
            println!("Can't find account for {:?}", agent_id);
            return false;
        };

        match order {
            LimitOrder::BidOrder { data } => {
                let maximum_transaction = data.price * data.size;

                if maximum_transaction.as_int + account_data.reserved_money.as_int
                    <= account_data.money.as_int
                {
                    self.accounts.get_mut(&agent_id).unwrap().reserved_money += maximum_transaction;
                    true
                } else {
                    false
                }
            }
            LimitOrder::AskOrder { data } => {
                let maximum_commodity_transfer = data.size;

                if maximum_commodity_transfer + account_data.reserved_commodity
                    <= account_data.commodity
                {
                    self.accounts.get_mut(&agent_id).unwrap().reserved_commodity +=
                        maximum_commodity_transfer;
                    true
                } else {
                    false
                }
            }
        }
    }

    fn process_agent_actions(&mut self) -> Vec<Order> {
        let mut rejected_orders: Vec<Order> = Vec::new();
        let mut agents: HashMap<AgentId, RefCell<Box<dyn Agent>>> = Default::default();

        std::mem::swap(&mut self.agents, &mut agents);

        agents.iter().for_each(|(agent_id, agent_ref)| {
            let orders: Vec<_> = agent_ref
                .borrow_mut()
                .produce_orders(&self.history)
                .into_iter()
                .map(|(side, price, size)| self.book.new_order_raw(side, Some(price), size, false))
                .collect();

            orders.iter().for_each(|&order| {
                let reserved = if let Result::Ok(limit_order) = order.try_into() {
                    self.reserve_limit(*agent_id, &limit_order)
                } else if let Result::Ok(market_order) = order.try_into() {
                    self.reserve_market(*agent_id, &market_order)
                } else {
                    false
                };

                if reserved {
                    let order_id = Into::<Order>::into(order).id;
                    self.order_map.insert(order_id, *agent_id);
                    self.book.add_order(order);
                } else {
                    rejected_orders.push(order);
                }
            });
        });

        std::mem::swap(&mut self.agents, &mut agents);

        rejected_orders
    }

    fn satisfy_transaction(&mut self, trns: &Transaction) {
        let Some(bidder_id) = self.order_map.get(&trns.bid_id) else {
            panic!("{:?} has no bidder", trns);
        };
        let Some(asker_id) = self.order_map.get(&trns.ask_id) else {
            panic!("{:?} has no asker", trns);
        };

        let Some(bidder_acc) = self.accounts.get_mut(bidder_id) else {
            panic!("bidder of {:?} has no account", trns);
        };

        bidder_acc.commodity += trns.size;
        bidder_acc.money -= trns.bid_loss;

        let Some(asker_acc) = self.accounts.get_mut(asker_id) else {
            panic!("asker of {:?} has no account", trns);
        };

        asker_acc.commodity -= trns.size;
        asker_acc.money += trns.ask_gain;
    }
}
