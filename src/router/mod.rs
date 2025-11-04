mod token_graph;

use crate::{router::token_graph::TokenGraph, uni_v2_pool::UniV2Pool};

use {itertools::Itertools as _, std::collections::HashMap};

#[derive(Debug)]
pub struct Router<'l> {
    /// Mapping token -> integer index
    token_index: HashMap<&'l str, usize>,
    /// Internal representation of the tokens and their pools relationships
    token_graph: TokenGraph,
}

impl Router<'_> {
    pub fn new(pools: Vec<UniV2Pool>) -> Self {
        let token_index = pools
            .iter()
            .flat_map(|p| [p.token0, p.token1])
            .unique()
            .enumerate()
            .map(|(i, token)| (token, i))
            .collect::<HashMap<_, _>>();

        let token_graph = TokenGraph::init(pools, &token_index);

        Router {
            token_index,
            token_graph,
        }
    }

    /// Solves for the maximum output amount of `output_token` that can be obtained by selling
    /// `input_amount` of `input_token`, updating the internal state of the router accordingly.
    pub fn solve(&mut self, input_token: &str, output_token: &str, input_amount: f64) -> f64 {
        let input_token = self.token_index[input_token];
        let output_token = self.token_index[output_token];

        self.token_graph
            .new_equilibrium_after_trade(input_token, output_token, input_amount)
    }
}
