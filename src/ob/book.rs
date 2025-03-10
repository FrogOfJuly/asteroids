use super::{
    amount::Amount,
    orders::{
        flat::{FlatOrder, OrderSide},
        limit::{AskLimitOrder, BidLimitOrder, LimitOrder},
        market::{AskMarketOrder, BidMarketOrder, MarketOrder},
    },
};
use std::{cell::RefCell, cmp::Ordering, collections::BinaryHeap};

#[derive(Clone, Debug, Default)]
pub struct OrderBook {
    limit_asks: BinaryHeap<AskLimitOrder>,
    limit_bids: BinaryHeap<BidLimitOrder>,

    market_asks: BinaryHeap<AskMarketOrder>,
    market_bids: BinaryHeap<BidMarketOrder>,

    id: RefCell<u64>,
    time: RefCell<i64>,
}

impl OrderBook {
    pub fn new_order(
        &self,
        side: OrderSide,
        price: Option<Amount>,
        size: i64,
        inc_time: bool,
    ) -> FlatOrder {
        let id = *self.id.borrow();
        *self.id.borrow_mut() += 1;

        let timestamp = *self.time.borrow();

        if inc_time {
            *self.time.borrow_mut() += 1;
        }

        FlatOrder {
            timestamp,
            id,
            side,
            price,
            size,
        }
    }

    pub fn clear_orders(&mut self) {
        self.limit_asks.clear();
        self.limit_bids.clear();

        self.market_asks.clear();
        self.market_bids.clear();
    }

    pub fn add_orders(&mut self, data: Vec<FlatOrder>) {
        data.into_iter().for_each(|o| self.add_order(o));
    }

    pub fn add_order(&mut self, order: FlatOrder) {
        if let Result::Ok(order) = order.try_into() {
            match order {
                MarketOrder::BidOrder { data } => self.market_bids.push(data.into()),
                MarketOrder::AskOrder { data } => self.market_asks.push(data.into()),
            };
        } else if let Result::Ok(order) = order.try_into() {
            match order {
                LimitOrder::BidOrder { data } => self.limit_bids.push(data.into()),
                LimitOrder::AskOrder { data } => self.limit_asks.push(data.into()),
            };
        }
    }

    pub fn from_orders(data: Vec<FlatOrder>) -> Self {
        let mut b = Self::default();
        b.add_orders(data);
        b
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Transaction {
    pub bid_id: u64,
    pub ask_id: u64,
    pub size: i64,
    pub bid_loss: Amount,
    pub ask_gain: Amount,
    pub diff: Amount,
}

impl OrderBook {
    pub fn match_all_market(&mut self, default_price: Option<Amount>) -> Vec<Transaction> {
        let mut transactions = Vec::new();

        while let Some(transaction) = self.match_ask_market_order() {
            transactions.push(transaction);
        }

        while let Some(transaction) = self.match_bid_market_order() {
            transactions.push(transaction);
        }

        let Some(default_price) = default_price else {
            return transactions;
        };

        while let Some(transaction) = self.match_market_orders(default_price) {
            transactions.push(transaction);
        }

        transactions
    }

    pub fn match_all_limit(&mut self) -> Vec<Transaction> {
        let mut transactions = Vec::new();
        while let Some(transaction) = self.match_limit_orders() {
            transactions.push(transaction);
        }
        transactions
    }

    pub fn all_orders(&self) -> Vec<FlatOrder> {
        let limit_asks: Vec<FlatOrder> = self
            .limit_asks
            .clone()
            .into_vec()
            .into_iter()
            .map(Into::into)
            .collect();
        let limit_bids: Vec<FlatOrder> = self
            .limit_bids
            .clone()
            .into_vec()
            .into_iter()
            .map(Into::into)
            .collect();

        let market_asks: Vec<FlatOrder> = self
            .market_asks
            .clone()
            .into_vec()
            .into_iter()
            .map(Into::into)
            .collect();

        let market_bids: Vec<FlatOrder> = self
            .market_bids
            .clone()
            .into_vec()
            .into_iter()
            .map(Into::into)
            .collect();

        [limit_asks, limit_bids, market_asks, market_bids].concat()
    }

    pub fn time_inc(&self) {
        *self.time.borrow_mut() += 1;
    }

    pub fn match_market_orders(&mut self, default_price: Amount) -> Option<Transaction> {
        let best_bid = self.market_bids.peek()?;
        let best_ask = self.market_asks.peek()?;

        let transaction_size = best_bid.data.size.min(best_ask.data.size);

        let bid_loss = default_price * transaction_size;
        let ask_gain = bid_loss;

        let transaction = Transaction {
            bid_id: best_bid.data.id,
            ask_id: best_ask.data.id,
            size: transaction_size,
            bid_loss,
            ask_gain,
            diff: Amount {
                as_int: bid_loss.as_int - ask_gain.as_int,
            },
        };

        let mut best_bid_mut = self.market_bids.pop()?;
        let mut best_ask_mut = self.market_asks.pop()?;

        if best_bid_mut.data.size > transaction_size {
            best_bid_mut.data.size -= transaction_size;
            self.market_bids.push(best_bid_mut);
        }

        if best_ask_mut.data.size > transaction_size {
            best_ask_mut.data.size -= transaction_size;
            self.market_asks.push(best_ask_mut);
        }

        if transaction.diff.as_int < 0 {
            panic!("{:?} gives net negative!", transaction)
        }

        Some(transaction)
    }

    pub fn match_ask_market_order(&mut self) -> Option<Transaction> {
        let market_order = self.market_asks.peek()?;
        let best_bid = self.limit_bids.peek()?;

        let transaction_size = market_order.data.size.min(best_bid.data.size);

        let ask_gain = best_bid.data.price * transaction_size;
        let bid_loss = ask_gain;

        let transaction = Transaction {
            bid_id: best_bid.data.id,
            ask_id: market_order.data.id,
            size: transaction_size,
            bid_loss,
            ask_gain,
            diff: Amount {
                as_int: bid_loss.as_int - ask_gain.as_int,
            },
        };

        let mut best_ask_mut = self.market_asks.pop()?;
        let mut best_bid_mut = self.limit_bids.pop()?;

        if best_ask_mut.data.size > transaction_size {
            best_ask_mut.data.size -= transaction_size;
            self.market_asks.push(best_ask_mut);
        }

        if best_bid_mut.data.size > transaction_size {
            best_bid_mut.data.size -= transaction_size;
            self.limit_bids.push(best_bid_mut);
        }

        Some(transaction)
    }
    pub fn match_bid_market_order(&mut self) -> Option<Transaction> {
        let market_order = self.market_bids.peek()?;
        let best_ask = self.limit_asks.peek()?;

        let transaction_size = market_order.data.size.min(best_ask.data.size);

        let ask_gain = best_ask.data.price * transaction_size;
        let bid_loss = ask_gain;

        let transaction = Transaction {
            bid_id: market_order.data.id,
            ask_id: best_ask.data.id,
            size: transaction_size,
            bid_loss,
            ask_gain,
            diff: Amount {
                as_int: bid_loss.as_int - ask_gain.as_int,
            },
        };

        let mut best_bid_mut = self.market_bids.pop()?;
        let mut best_ask_mut = self.limit_asks.pop()?;

        if best_bid_mut.data.size > transaction_size {
            best_bid_mut.data.size -= transaction_size;
            self.market_bids.push(best_bid_mut);
        }

        if best_ask_mut.data.size > transaction_size {
            best_ask_mut.data.size -= transaction_size;
            self.limit_asks.push(best_ask_mut);
        }

        Some(transaction)
    }

    pub fn match_limit_orders(&mut self) -> Option<Transaction> {
        let best_bid = self.limit_bids.peek()?;
        let best_ask = self.limit_asks.peek()?;

        if let Ordering::Less = best_bid.data.price.cmp(&best_ask.data.price) {
            return None;
        }

        let transaction_size = best_bid.data.size.min(best_ask.data.size);
        let bid_loss = best_bid.data.price * transaction_size;
        let ask_gain = best_ask.data.price * transaction_size;

        let transaction = Transaction {
            bid_id: best_bid.data.id,
            ask_id: best_ask.data.id,
            size: transaction_size,
            bid_loss,
            ask_gain,
            diff: Amount {
                as_int: bid_loss.as_int - ask_gain.as_int,
            },
        };

        let mut best_bid_mut = self.limit_bids.pop()?;
        let mut best_ask_mut = self.limit_asks.pop()?;

        if best_bid_mut.data.size > transaction_size {
            best_bid_mut.data.size -= transaction_size;
            self.limit_bids.push(best_bid_mut);
        }

        if best_ask_mut.data.size > transaction_size {
            best_ask_mut.data.size -= transaction_size;
            self.limit_asks.push(best_ask_mut);
        }

        if transaction.diff.as_int < 0 {
            panic!("{:?} gives net negative!", transaction)
        }

        Some(transaction)
    }
}

#[cfg(test)]
mod prop_tests {
    use super::*;
    use quickcheck_macros::quickcheck;

    impl quickcheck::Arbitrary for OrderSide {
        fn arbitrary(g: &mut quickcheck::Gen) -> OrderSide {
            g.choose(&[OrderSide::Bid, OrderSide::Ask])
                .cloned()
                .unwrap()
        }
    }

    impl quickcheck::Arbitrary for Amount {
        fn arbitrary(g: &mut quickcheck::Gen) -> Amount {
            Amount {
                as_int: *g.choose(Vec::from_iter(1..100).as_slice()).unwrap(),
            }
        }
    }

    #[quickcheck]
    fn match_all_limit_idempotent(order_data: Vec<(OrderSide, Amount, u32)>) -> bool {
        let mut ob = OrderBook::default();

        let orders: Vec<_> = order_data
            .into_iter()
            .map(|(side, price, size)| ob.new_order(side, Some(price), size as i64, false))
            .collect();

        orders.into_iter().for_each(|order| ob.add_order(order));

        // print!("{:?} -> ", ob.all_orders().len());

        ob.match_all_limit();

        let remaining = ob.all_orders();

        // println!("{:?}", ob.all_orders().len());

        ob.match_all_limit();

        remaining == ob.all_orders()
    }

    #[quickcheck]
    fn market_matches_all_limits(order_data: Vec<(OrderSide, Amount, Amount)>) -> bool {
        if order_data.is_empty() {
            return true;
        }

        // println!("order data: {:?}", order_data);

        // println!("creating ob..");
        let mut ob = OrderBook::default();

        // println!("creating orders..");
        let orders: Vec<_> = order_data
            .iter()
            .map(|(side, price, size)| ob.new_order(*side, Some(*price), size.as_int, false))
            .collect();

        // println!("adding orders..");
        orders.into_iter().for_each(|order| ob.add_order(order));

        // print!("counting amounts..");
        let (bid_amount, ask_amount) = order_data.iter().fold((0, 0), |(l, r), d| match d.0 {
            OrderSide::Bid => (l + d.2.as_int, r),
            OrderSide::Ask => (l, r + d.2.as_int),
        });

        // println!("counted: {}, {}", bid_amount, ask_amount);

        // println!("creating market orders..");

        let market_ask = ob.new_order(OrderSide::Ask, None, bid_amount, false);
        let market_bid = ob.new_order(OrderSide::Bid, None, ask_amount, false);

        // println!("adding market orders..");

        if bid_amount > 0 {
            // println!("ask_order: {:?}", market_ask);
            ob.add_order(market_ask)
        };
        if ask_amount > 0 {
            // println!("bid_order: {:?}", market_bid);
            ob.add_order(market_bid)
        };

        // println!("matching..");
        // print!("{:?} -> ", ob.all_orders().len());

        ob.match_all_market(None);

        // println!("{:?}", ob.all_orders().len());

        // println!("remaining: {:?}", ob.all_orders());

        ob.all_orders().is_empty()
    }
}

#[cfg(test)]
mod limit_tests {
    use super::*;

    #[test]
    fn match_1v1_exact() {
        let mut ob = OrderBook::default();

        [
            (OrderSide::Ask, Amount { as_int: 1 }, 1),
            (OrderSide::Bid, Amount { as_int: 1 }, 1),
        ]
        .map(|(side, price, size)| ob.new_order(side, Some(price), size, false))
        .into_iter()
        .for_each(|order| ob.add_order(order));

        let transactions = ob.match_all_limit();

        assert_eq!(ob.limit_asks.len(), 0);
        assert_eq!(ob.limit_bids.len(), 0);
        assert_eq!(transactions.len(), 1);
        assert_eq!(
            transactions.first(),
            Some(&Transaction {
                bid_id: 1,
                ask_id: 0,
                size: 1,
                bid_loss: Amount { as_int: 1 },
                ask_gain: Amount { as_int: 1 },
                diff: Amount { as_int: 0 }
            })
        );
    }

    #[test]
    fn match_1v1_fail() {
        let mut ob = OrderBook::default();

        [
            (OrderSide::Ask, Amount { as_int: 2 }, 1),
            (OrderSide::Bid, Amount { as_int: 1 }, 1),
        ]
        .map(|(side, price, size)| ob.new_order(side, Some(price), size, false))
        .into_iter()
        .for_each(|order| ob.add_order(order));

        let transactions = ob.match_all_limit();

        assert_eq!(ob.limit_asks.len(), 1);
        assert_eq!(ob.limit_bids.len(), 1);
        assert_eq!(transactions.len(), 0);
    }

    #[test]
    fn match_1v1_diff() {
        let mut ob = OrderBook::default();

        [
            (OrderSide::Ask, Amount { as_int: 1 }, 1),
            (OrderSide::Bid, Amount { as_int: 2 }, 1),
        ]
        .map(|(side, price, size)| ob.new_order(side, Some(price), size, false))
        .into_iter()
        .for_each(|order| ob.add_order(order));

        let transactions = ob.match_all_limit();

        assert_eq!(ob.limit_asks.len(), 0);
        assert_eq!(ob.limit_bids.len(), 0);
        assert_eq!(transactions.len(), 1);
        assert_eq!(
            transactions.first(),
            Some(&Transaction {
                bid_id: 1,
                ask_id: 0,
                size: 1,
                bid_loss: Amount { as_int: 2 },
                ask_gain: Amount { as_int: 1 },
                diff: Amount { as_int: 1 }
            })
        );
    }

    #[test]
    fn match_1v2_exact() {
        let mut ob = OrderBook::default();

        [
            (OrderSide::Ask, Amount { as_int: 1 }, 1),
            (OrderSide::Ask, Amount { as_int: 1 }, 1),
            (OrderSide::Bid, Amount { as_int: 1 }, 2),
        ]
        .map(|(side, price, size)| ob.new_order(side, Some(price), size, true))
        .into_iter()
        .for_each(|order| ob.add_order(order));

        let transactions = ob.match_all_limit();

        assert_eq!(ob.limit_asks.len(), 0);
        assert_eq!(ob.limit_bids.len(), 0);
        assert_eq!(transactions.len(), 2);

        assert_eq!(
            transactions.first(),
            Some(&Transaction {
                bid_id: 2,
                ask_id: 0,
                size: 1,
                bid_loss: Amount { as_int: 1 },
                ask_gain: Amount { as_int: 1 },
                diff: Amount { as_int: 0 }
            })
        );

        assert_eq!(
            transactions.get(1),
            Some(&Transaction {
                bid_id: 2,
                ask_id: 1,
                size: 1,
                bid_loss: Amount { as_int: 1 },
                ask_gain: Amount { as_int: 1 },
                diff: Amount { as_int: 0 }
            })
        );
    }

    #[test]
    fn match_2v1_exact() {
        let mut ob = OrderBook::default();

        [
            (OrderSide::Ask, Amount { as_int: 1 }, 2),
            (OrderSide::Bid, Amount { as_int: 1 }, 1),
            (OrderSide::Bid, Amount { as_int: 1 }, 1),
        ]
        .map(|(side, price, size)| ob.new_order(side, Some(price), size, true))
        .into_iter()
        .for_each(|order| ob.add_order(order));

        let transactions = ob.match_all_limit();

        assert_eq!(ob.limit_asks.len(), 0);
        assert_eq!(ob.limit_bids.len(), 0);
        assert_eq!(transactions.len(), 2);

        assert_eq!(
            transactions.first(),
            Some(&Transaction {
                bid_id: 1,
                ask_id: 0,
                size: 1,
                bid_loss: Amount { as_int: 1 },
                ask_gain: Amount { as_int: 1 },
                diff: Amount { as_int: 0 }
            })
        );

        assert_eq!(
            transactions.get(1),
            Some(&Transaction {
                bid_id: 2,
                ask_id: 0,
                size: 1,
                bid_loss: Amount { as_int: 1 },
                ask_gain: Amount { as_int: 1 },
                diff: Amount { as_int: 0 }
            })
        );
    }
}
