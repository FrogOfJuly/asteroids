#![feature(trait_alias)]

use std::cell::RefCell;

use agents::{BuyAgent, SellAgent};
use market::{
    agent::Agent as GenericAgent,
    amount::Amount,
    market::{History, Market, MarketInfo},
};

enum CommodityType {
    Unit,
}

trait Agent =
    GenericAgent<CommodityType = CommodityType, MarketInfoType = MarketInfo<CommodityType>>;

fn main() {
    let mut market = Market::new(MarketInfo {
        name: "test".to_owned(),
        commodity: CommodityType::Unit,
    });

    let buy_agent: Box<dyn Agent> = Box::new(BuyAgent::<_> {
        ask_size: 1,
        ask_amount: 2,
        period: 2,
        eigen_price: Some(Amount { as_int: 10 }),
        _ph: Default::default(),
    });

    let sell_agent: Box<dyn Agent> = Box::new(SellAgent::<_> {
        ask_size: 1,
        ask_amount: 2,
        period: 2,
        eigen_price: Some(Amount { as_int: 8 }),
        _ph: Default::default(),
    });

    let agents = [buy_agent, sell_agent]
        .map(|agent| (market.register_with_default_acc(), RefCell::new(agent)));

    market.accounts.get_mut(&agents[0].0).unwrap().money += Amount { as_int: 30 };
    market.accounts.get_mut(&agents[1].0).unwrap().commodity += 20;

    let mut history = History::default();

    for step in 1..=10 {
        let bidder_money = market.accounts.get(&agents[0].0).unwrap().money.as_int;
        let asker_comm = market.accounts.get(&agents[1].0).unwrap().commodity;

        println!("history: {}", history);
        println!("-------");

        let rejected_orders = market.agents_submit_orders(agents.as_slice(), &history);
        let transactions = market.process_submitted_orders(history.market_price());
        let unfulfilled_orders = market.all_orders();

        history = History {
            step,
            transactions,
            rejected_orders,
            unfulfilled_orders,
        };

        market.clear_orders();

        println!(
            "> buyer money: {bidder_money}->{:?}",
            market.accounts.get(&agents[0].0).unwrap().money.as_int
        );
        println!(
            "> seller comm: {asker_comm}->{:?}",
            market.accounts.get(&agents[1].0).unwrap().commodity
        );

        market.accounts.get_mut(&agents[0].0).unwrap().money += Amount { as_int: 5 };
        market.accounts.get_mut(&agents[1].0).unwrap().commodity += 2;
    }
}
