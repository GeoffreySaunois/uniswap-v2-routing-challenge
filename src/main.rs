use crate::{router::Router, uni_v2_pool::UniV2Pool};

mod router;
mod uni_v2_pool;

fn main() {
    let pools: Vec<UniV2Pool> = vec![
        UniV2Pool::new("A", "B", 10., 40.),
        UniV2Pool::new("A", "B", 10., 40.),
    ];
    let mut router = Router::new(pools);
    let a_sell_amount = 20.;

    println!("Initial router state: {router:#?}");
    let b_output_amount = router.solve("A", "B", a_sell_amount);
    println!("Final router state: {router:#?}");
    println!("Solution for {a_sell_amount:.2} A to B: {b_output_amount:.2}");

    let pools = vec![
        UniV2Pool::new("ETH", "USDC", 2_000., 2_000_000.),
        UniV2Pool::new("ETH", "USDC", 1_000., 1_000_000.),
        UniV2Pool::new("ETH", "DAI", 1_000., 900_000.),
        UniV2Pool::new("ETH", "DAI", 3_000., 2_800_000.),
        UniV2Pool::new("ETH", "DAI", 3_000., 3_100_000.),
        UniV2Pool::new("DAI", "USDC", 1_000_000., 1_000_000.),
        UniV2Pool::new("DAI", "USDC", 2_000_000., 2_000_000.),
        UniV2Pool::new("DAI", "USDT", 1_000_000., 900_000.),
        UniV2Pool::new("DAI", "USDT", 900_000., 1_000_000.),
        UniV2Pool::new("ETH", "USDT", 2_000., 2_000_000.),
        UniV2Pool::new("ETH", "USDT", 10_000., 10_000_000.),
    ];

    let mut router = Router::new(pools);

    // First trade before equilibrium, we'll win extra tokens from arbitrage
    let eth_sell_amount = 10.;
    let usdc_output_amount = router.solve("ETH", "USDC", eth_sell_amount);
    println!("Solution for {eth_sell_amount:.2} ETH to USDC: {usdc_output_amount:.2}");

    // Second trade after equilibrium, now the conversions are fair
    let usdc_sell_amount = 10000.;
    let eth_output_amount = router.solve("USDC", "ETH", usdc_sell_amount);
    println!("Solution for {usdc_sell_amount:.2} USDC to ETH: {eth_output_amount:.2}");
}
