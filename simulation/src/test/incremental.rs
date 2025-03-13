use std::cell::RefCell;

use agents::{BuyAgent, IncBuyAgent, IncSellAgent, SellAgent};
use market::{
    account::Account,
    agent::{Agent as GenericAgent, AgentId},
    amount::Amount,
    market::{History, Market, MarketInfo},
};

enum CommodityType {
    Unit,
}

trait Agent =
    GenericAgent<CommodityType = CommodityType, MarketInfoType = MarketInfo<CommodityType>>;

#[test]
fn run() {
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
        .take(10)
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
        .take(10)
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
        // println!("id: {:?}", id);
        agent.setup(id, &market.info);
        (id, RefCell::new(agent))
    })
    .collect();

    let mut history = History::default();
    let mut market_price = None;

    
    for step in 1..=100 {
        // println!("history: {}", history);
        // println!("-------");

        let rejected_orders = market.agents_submit_orders(agents.as_slice(), &history);
        let transactions = market.process_submitted_orders(market_price);
        let unfulfilled_orders = market.all_orders();

        history = History {
            step,
            transactions,
            rejected_orders,
            unfulfilled_orders,
        };

        if !history.no_transactions() {
            market_price = history.market_price()
        }

        println!(
            "market price: {}",
            market_price.map_or("?".to_owned(), |x| x.as_int.to_string())
        );

        market.clear_reserves_and_orders();
    }
}
