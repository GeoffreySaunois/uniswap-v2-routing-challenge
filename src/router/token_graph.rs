use crate::uni_v2_pool::UniV2Pool;

use std::collections::HashMap;

const TOLERANCE: f64 = 1e-12;
const MAX_ITERS: usize = 20_000;

#[derive(Debug)]
pub(super) struct TokenGraph {
    nodes: Vec<TokenNode>,
}

/// Represents a node in the token–liquidity graph used by the router.
///
/// Each `TokenNode` corresponds to a single token in the system and carries:
/// - its **total reserve** across all pools in which it appears.
/// - its **square-root price variable** `q` used in the equilibrium solver.
/// - and a map of **adjacent tokens** with their aggregated *geometric liquidities*.
///
/// ### Pool aggregation
///
/// All Uniswap-V2 pools connecting the same token pair `(u,v)` are collapsed
/// into a single effective edge in the graph, whose liquidity is the sum of the
/// square roots of their individual invariants:
///
/// `K(u, v) = Σ √kᵢ`, over all pools `i` linking `u` and `v`,
///
/// The resulting structure is a **aggregated, undirected, and weighted graph of
/// tokens**, where each edge encodes the combined liquidity between two tokens.
#[derive(Debug, Clone)]
pub(super) struct TokenNode {
    /// Total amount of the token across all pools
    total_reserve: f64,
    /// Square-root price of the token, used for equilibrium computation
    q: f64,
    /// Adjacent tokens and the associated geometric liquidities
    adjacents_token: HashMap<usize, f64>,
}

impl TokenGraph {
    /// Initializes the aggregated token graph from pools:
    /// - accumulates Tₜ (totals per token),
    /// - sums K(u, v) = Σ √kᵢ.
    /// - sets initial prices `q` to 1.0 for all tokens (those prices will be updated during first
    ///   equilibrium computation).
    pub(super) fn from_pools(pools: Vec<UniV2Pool>, token_index: &HashMap<&str, usize>) -> Self {
        let mut nodes = vec![
            TokenNode {
                total_reserve: 0.0,
                q: 1.0,
                adjacents_token: HashMap::new(),
            };
            token_index.len()
        ];

        for pool in pools {
            let index_0 = token_index[pool.token0];
            let index_1 = token_index[pool.token1];
            nodes[index_0].total_reserve += pool.reserve0;
            nodes[index_1].total_reserve += pool.reserve1;
            let liquidity = (pool.reserve0 * pool.reserve1).sqrt();
            *nodes[index_0].adjacents_token.entry(index_1).or_insert(0.0) += liquidity;
            *nodes[index_1].adjacents_token.entry(index_0).or_insert(0.0) += liquidity;
        }

        Self { nodes }
    }

    /// Computes the maximum amount of `output_token` obtainable by swapping
    /// `input_amount` of `input_token`, and updates the router’s internal state.
    ///
    /// Conceptually, this adds `input_amount` to the total reserves of `input_token`,
    /// then resolves the new no-arbitrage equilibrium of the entire pool network.
    /// The difference in `output_token` reserves before and after equilibrium
    /// represents the extractable output amount.
    pub(super) fn apply_trade_and_solve(
        &mut self,
        input_token: usize,
        output_token: usize,
        input_amount: f64,
    ) -> f64 {
        self.nodes[input_token].total_reserve += input_amount;
        let output_amount = self.no_arbitrage_equilibrium(output_token);

        output_amount
    }
}

impl TokenGraph {
    /// Fixed-point no-arbitrage solver (Gauss–Seidel), computing the maximum amount of
    /// `output_token` `f` that can be extracted in the process.
    ///
    /// The system enforces the conservation of total token balances:
    ///
    /// ```text
    ///   ∑ K(u, v) * (q_u / q_v) = T_u,   for all u ≠ output_token
    /// ```
    ///
    /// where:
    /// - `K(u, v) = Σ √kᵢ`  is the geometric liquidity between `u` and `v` (sum over all pools i
    ///   linking `u` and `v`)
    /// - `T_u` is the total reserve of token `u` across all pools
    /// - `q_u` is the sqrt-price variable for token `u`
    ///
    /// The fixed-point iteration updates until convergence (`max_relative_change < TOLERANCE`):
    ///
    /// ```text
    ///   q_u ← T_u / ( ∑ K(u, v) / q_v ),   for all u ≠ output_token
    /// ```
    ///
    /// Once the prices `q` have converged, the post-equilibrium total of the `output_token` is:
    ///
    /// ```text
    ///   T'_f = ∑ K(f, v) * (q_f / q_v),
    /// ```
    ///
    /// and the amount of `output_token` that can be extracted is:
    ///
    /// ```text
    ///   Δf = T_f − T'_f.
    /// ```
    ///
    /// Complexity:  `O(MAX_ITERS × E)`, where E is the number of edges in the token graph.
    fn no_arbitrage_equilibrium(&mut self, output_token: usize) -> f64 {
        for _ in 0..MAX_ITERS {
            let mut max_relative_change = 0.0;

            for token in 0..self.nodes.len() {
                // Skip the output token
                if token == output_token {
                    continue;
                }
                let q = self.nodes[token].q;

                // Update q_u ← T_u / ( ∑ K(u, v) / q_v )
                let mut denom = 0.0;
                for (paired_token, liquidity) in self.neighbors_with_liquidity(token) {
                    denom += liquidity / self.nodes[paired_token].q;
                }
                let updated_q = self.nodes[token].total_reserve / denom;

                let relative_change = ((updated_q - q).abs()) / q;

                if relative_change > max_relative_change {
                    max_relative_change = relative_change;
                }

                self.nodes[token].q = updated_q;
            }

            if max_relative_change < TOLERANCE {
                break;
            }
        }

        // Optional renormalization: keep first token as price reference
        self.normalize_prices();

        // Compute and update the output reserve based after equilibrium
        let mut output_reserve = 0.0;
        for (token, liquidity) in self.neighbors_with_liquidity(output_token) {
            output_reserve += liquidity * (self.nodes[output_token].q / self.nodes[token].q);
        }
        let extracted_amount = self.nodes[output_token].total_reserve - output_reserve;
        self.nodes[output_token].total_reserve = output_reserve;

        extracted_amount
    }

    /// Returns an iterator over the neighboring tokens and their associated geometric liquidities.
    fn neighbors_with_liquidity(&self, token: usize) -> impl Iterator<Item = (usize, f64)> {
        self.nodes[token]
            .adjacents_token
            .iter()
            .map(|(&k, &v)| (k, v))
    }

    /// Renormalizes all root prices `q` so that the first token has price 1.0.
    fn normalize_prices(&mut self) {
        const REFERENCE_TOKEN: usize = 0;
        let ref_q = self.nodes[REFERENCE_TOKEN].q;
        for node in &mut self.nodes {
            node.q /= ref_q;
        }
    }
}
