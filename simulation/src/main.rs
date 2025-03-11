#![feature(trait_alias)]

use agents::{BuyAgent, SellAgent};
use market::{
    mkt::{
        agent::Agent as GeneralAgent,
        market::{Market, MarketInfo},
    },
    ob::amount::Amount,
};

enum CommodityType {
    Unit,
}

trait Agent =
    GeneralAgent<CommodityType = CommodityType, MarketInfoType = MarketInfo<CommodityType>>;

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

    let ids = [buy_agent, sell_agent].map(|agent| market.register_with_default_acc(agent));

    market.accounts.get_mut(&ids[0]).unwrap().money += Amount { as_int: 30 };
    market.accounts.get_mut(&ids[1]).unwrap().commodity += 20;

    for _ in 1..10 {

        let bidder_money = market.accounts.get(&ids[0]).unwrap().money.as_int;
        let asker_comm = market.accounts.get(&ids[1]).unwrap().commodity;

        market.clear_orders();
        market.step();
        market.history.inc_step();

        println!("-------");

        println!(
            "> buyer money: {bidder_money}->{:?}",
            market.accounts.get(&ids[0]).unwrap().money.as_int
        );
        println!(
            "> seller comm: {asker_comm}->{:?}",
            market.accounts.get(&ids[1]).unwrap().commodity
        );

        println!("{}", market.history);

        market.accounts.get_mut(&ids[0]).unwrap().money += Amount { as_int: 5 };
        market.accounts.get_mut(&ids[1]).unwrap().commodity += 2;
    }
}
