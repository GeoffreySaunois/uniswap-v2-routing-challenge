# Swaap routing challenge

## Context

A Uniswap V2 pool functions as an on-chain market-making engine, enabling the permissionless exchange of two tokens.

The system is governed by the constant product formula, which always hold:

`reserve0 * reserve1 = k`

where `reserve0` (respectively `reserve1`) represents the available liquidity of `token0` (respectively `token1`) in the pool.

This fundamental equation ensures that the pool never depletes either asset, as the price dynamically adjusts — approaching infinity as one reserve approaches zero.
To determine the `output_amount` of `output_token` received when swapping a given `input_amount` of `input_token`, the following equation must be solved:

`(reserve_in + input_amount) * (reserve_out − output_amount) = k = reserve_in * reserve_out`

This equation maintains the constant product invariant while reflecting the impact of each trade on price and liquidity.

This is implemented in [src/uni_v2_pool.rs](src/uni_v2_pool.rs).

## Goal

Given a list of Uniswap V2 pools, an input token, an input amount and an output token, compute the maximum amount of output tokens that can be received. This will involve traversing multiple pools.

**Implement the `solve()` function in [src/router.rs](src/router.rs) and test your solution with `cargo run`.**
