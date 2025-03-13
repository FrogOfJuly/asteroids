use std::{cell::RefCell, collections::HashMap, fmt::Display, iter::repeat};

use crate::{
    amount::Amount,
    book::{OrderBook, Transaction},
    orders::{
        flat::{Order, OrderData},
        limit::LimitOrder,
    },
};

use super::{
    account::Account,
    agent::{Agent, AgentId},
};

#[derive(Default, Debug)]
pub struct History {
    pub step: u64,
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

    pub fn clear(&mut self) {
        self.transactions = Default::default();
        self.rejected_orders = Default::default();
        self.unfulfilled_orders = Default::default();
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

type AgentType<T> = Box<dyn Agent<CommodityType = T, MarketInfoType = MarketInfo<T>>>;
type AgentRefType<T> = RefCell<AgentType<T>>;

pub struct MarketInfo<CommodityType> {
    pub name: String,
    pub commodity: CommodityType,
}

pub struct Market<CommodityType> {
    pub info: MarketInfo<CommodityType>,

    book: OrderBook,

    id: RefCell<u64>,

    market_account: Account,

    pub accounts: HashMap<AgentId, Account>,
    order_map: HashMap<u64, AgentId>,
}

impl<CommodityType> Market<CommodityType> {
    pub type AgentRefType = AgentRefType<CommodityType>;
    pub type AgentType = AgentRefType<CommodityType>;

    pub fn new(info: MarketInfo<CommodityType>) -> Market<CommodityType> {
        Self {
            book: Default::default(),
            info,
            id: Default::default(),
            market_account: Default::default(),
            accounts: Default::default(),
            order_map: Default::default(),
        }
    }

    pub fn all_orders(&self) -> Vec<Order> {
        self.book.all_orders()
    }

    pub fn create_orders(
        &self,
        submitter: &AgentId,
        submissions: &[OrderData],
    ) -> Vec<Option<Order>> {
        submissions
            .iter()
            .map(|data| {
                self.accounts
                    .get(submitter)
                    .and(self.book.new_order_checked(*data))
            })
            .collect()
    }

    pub fn submit_order(&mut self, submitter: &AgentId, order: Order) -> Option<Order> {
        let reserved = self
            .accounts
            .get_mut(submitter)
            .unwrap()
            .reserve_order(order);

        if reserved {
            let order_id = Into::<Order>::into(order).id;
            self.order_map.insert(order_id, *submitter);
            self.book.add_order(order);
            None
        } else {
            Some(order)
        }
    }

    pub fn register_with_acc(&mut self, account: Account) -> AgentId {
        let id = AgentId::new(*self.id.borrow());
        *self.id.borrow_mut() += 1;
        self.accounts.insert(id, account);
        id
    }

    pub fn register_with_starting_acc(&mut self) -> AgentId {
        self.register_with_acc(Account::starting_account())
    }

    pub fn register_with_default_acc(&mut self) -> AgentId {
        self.register_with_acc(Account::default())
    }

    pub fn owner(&self, order: &LimitOrder) -> Option<AgentId> {
        let order_id = Into::<Order>::into(*order).id;
        self.order_map.get(&order_id).cloned()
    }

    pub fn account(&self, agent_id: AgentId) -> Option<Account> {
        self.accounts.get(&agent_id).cloned()
    }

    pub fn clear_orders(&mut self) {
        self.order_map.clear();
        self.clear_reservations();
        self.book.clear_orders();
    }

    pub fn agents_submit_orders(
        &mut self,
        agents: &[(AgentId, Self::AgentRefType)],
        history: &History,
    ) -> Vec<Order> {
        let orders: Vec<(Option<Order>, AgentId)> = agents
            .iter()
            .filter_map(|(id, agent)| self.account(*id).map(|account| (id, agent, account)))
            .flat_map(|(id, agent, account)| {
                let data = (*agent.borrow_mut()).produce_orders(&account, &self.info, history);

                self.create_orders(id, data.as_slice())
                    .into_iter()
                    .zip(repeat(*id))
            })
            .collect();

        orders
            .into_iter()
            .flat_map(|(order, id)| order.map(|order| self.submit_order(&id, order)))
            .flatten()
            .collect()
    }

    pub fn process_submitted_orders(
        &mut self,
        prev_market_price: Option<Amount>,
    ) -> Vec<Transaction> {
        // Assumes clean history

        let primary_market_transactions = self.book.match_all_market(None);

        primary_market_transactions
            .iter()
            .for_each(|trns| self.fulfill_transaction(trns));

        // determine new market price
        let market_price = if primary_market_transactions.is_empty() {
            None
        } else {
            Some(
                primary_market_transactions
                    .iter()
                    .map(|tr| (tr.ask_gain + tr.bid_loss).as_int / 2)
                    .sum(),
            )
        }
        .map(|as_int| Amount { as_int });

        //if there are market transactions left, match and fullfil them with either
        // * current market price
        // * previous market price
        let secondary_market_transactions = self
            .book
            .match_all_market(market_price.or(prev_market_price));

        secondary_market_transactions
            .iter()
            .for_each(|trns| self.fulfill_transaction(trns));

        //if there are limit transactions left, match and fullfil as much as possible

        let limit_transactions = self.book.match_all_limit();

        limit_transactions
            .iter()
            .for_each(|trns| self.fulfill_transaction(trns));

        // record all the history for next step

        [
            primary_market_transactions,
            secondary_market_transactions,
            limit_transactions,
        ]
        .concat()
    }
}

impl<CommodityType> Market<CommodityType> {
    fn clear_reservations(&mut self) {
        self.accounts.iter_mut().for_each(|(_, acc_mut)| {
            acc_mut.reserved_commodity = Default::default();
            acc_mut.reserved_money = Default::default();
        });
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
