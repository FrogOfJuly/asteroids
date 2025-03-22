use crate::configurations::example1::MarketConfiguration;

#[test]
fn run() {
    let mut conf = MarketConfiguration::new();
    for _ in 1..=10 {
        let market_price = conf.step();

        println!(
            "market price: {}",
            market_price.map_or("?".to_owned(), |x| x.as_int.to_string())
        );
    }
}
