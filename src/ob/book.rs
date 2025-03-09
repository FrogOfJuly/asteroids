use super::order::{Amount, AskLimitOrder, BidLimitOrder, FlatLimitOrder, LimitOrder, OrderSide};
use std::{cell::RefCell, cmp::Ordering, collections::BinaryHeap};

#[derive(Clone, Debug, Default)]
pub struct OrderBook {
    asks: BinaryHeap<AskLimitOrder>,
    bids: BinaryHeap<BidLimitOrder>,

    id: RefCell<u64>,
    time: RefCell<i64>,
}

impl OrderBook {
    pub fn new_order(
        &self,
        side: OrderSide,
        price: Amount,
        size: i64,
        inc_time: bool,
    ) -> LimitOrder {
        let id = *self.id.borrow();
        *self.id.borrow_mut() += 1;

        let timestamp = *self.time.borrow();

        if inc_time {
            *self.time.borrow_mut() += 1;
        }

        FlatLimitOrder {
            timestamp,
            id,
            side,
            price,
            size,
        }
        .into()
    }

    pub fn clear_orders(&mut self) {
        self.asks.clear();
        self.bids.clear();
    }

    pub fn add_orders(&mut self, data: Vec<LimitOrder>) {
        data.into_iter().for_each(|o| self.add_order(o));
    }

    pub fn add_order(&mut self, order: LimitOrder) {
        match order {
            LimitOrder::BidOrder { data } => self.bids.push(data.into()),
            LimitOrder::AskOrder { data } => self.asks.push(data.into()),
        };
    }

    pub fn from_orders(data: Vec<LimitOrder>) -> Self {
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
    // might include commission ratio and fees
    pub fn match_order(&mut self) -> Option<Transaction> {
        let best_bid = self.bids.peek()?;
        let best_ask = self.asks.peek()?;

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

        let mut best_bid_mut = self.bids.pop()?;
        let mut best_ask_mut = self.asks.pop()?;

        if best_bid_mut.data.size > transaction_size {
            best_bid_mut.data.size -= transaction_size;
            self.bids.push(best_bid_mut);
        }

        if best_ask_mut.data.size > transaction_size {
            best_ask_mut.data.size -= transaction_size;
            self.asks.push(best_ask_mut);
        }

        if transaction.diff.as_int < 0 {
            panic!("{:?} gives net negative!", transaction)
        }

        Some(transaction)
    }

    pub fn match_all(&mut self) -> Vec<Transaction> {
        let mut transactions = Vec::new();
        while let Some(transaction) = self.match_order() {
            transactions.push(transaction);
        }
        transactions
    }

    pub fn time_inc(&self) {
        *self.time.borrow_mut() += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn match_1v1_exact() {
        let mut ob = OrderBook::default();

        [
            (OrderSide::Ask, Amount { as_int: 1 }, 1),
            (OrderSide::Bid, Amount { as_int: 1 }, 1),
        ]
        .map(|(side, price, size)| ob.new_order(side, price, size, false))
        .into_iter()
        .for_each(|order| ob.add_order(order));

        let transactions = ob.match_all();

        assert_eq!(ob.asks.len(), 0);
        assert_eq!(ob.bids.len(), 0);
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
        .map(|(side, price, size)| ob.new_order(side, price, size, false))
        .into_iter()
        .for_each(|order| ob.add_order(order));

        let transactions = ob.match_all();

        assert_eq!(ob.asks.len(), 1);
        assert_eq!(ob.bids.len(), 1);
        assert_eq!(transactions.len(), 0);
    }

    #[test]
    fn match_1v1_diff() {
        let mut ob = OrderBook::default();

        [
            (OrderSide::Ask, Amount { as_int: 1 }, 1),
            (OrderSide::Bid, Amount { as_int: 2 }, 1),
        ]
        .map(|(side, price, size)| ob.new_order(side, price, size, false))
        .into_iter()
        .for_each(|order| ob.add_order(order));

        let transactions = ob.match_all();

        assert_eq!(ob.asks.len(), 0);
        assert_eq!(ob.bids.len(), 0);
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
        .map(|(side, price, size)| ob.new_order(side, price, size, true))
        .into_iter()
        .for_each(|order| ob.add_order(order));

        let transactions = ob.match_all();

        assert_eq!(ob.asks.len(), 0);
        assert_eq!(ob.bids.len(), 0);
        assert_eq!(transactions.len(), 2);

        assert_eq!(
            transactions.first(),
            Some(&Transaction {
                bid_id: 2,
                ask_id: 1,
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

    #[test]
    fn match_2v1_exact() {
        let mut ob = OrderBook::default();

        [
            (OrderSide::Ask, Amount { as_int: 1 }, 2),
            (OrderSide::Bid, Amount { as_int: 1 }, 1),
            (OrderSide::Bid, Amount { as_int: 1 }, 1),
        ]
        .map(|(side, price, size)| ob.new_order(side, price, size, true))
        .into_iter()
        .for_each(|order| ob.add_order(order));

        let transactions = ob.match_all();

        assert_eq!(ob.asks.len(), 0);
        assert_eq!(ob.bids.len(), 0);
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
                bid_id: 1,
                ask_id: 0,
                size: 1,
                bid_loss: Amount { as_int: 1 },
                ask_gain: Amount { as_int: 1 },
                diff: Amount { as_int: 0 }
            })
        );
    }
}
