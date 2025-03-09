use ob::book::OrderBook;

pub mod ob;

fn main() {
    let mut ob = OrderBook::default();
    ob.clear_orders();
}
