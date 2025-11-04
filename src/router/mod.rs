mod liquidities;

use liquidities::PairwiseLiquidities;

use crate::uni_v2_pool::UniV2Pool;

use {itertools::Itertools as _, std::collections::HashMap};

const TOLERANCE: f64 = 1e-12;
const MAX_ITERS: usize = 20_000;

#[derive(Debug)]
pub struct Router {
    /// Number of distinct tokens in the graph
    n_tokens: usize,
    /// Mapping token -> integer index
    token_index: HashMap<&'static str, usize>,
    /// Total amount of each token across all pools
    reserve_by_token: Vec<f64>,
    /// Square-root prices per token, used for equilibrium computation
    q_by_token: Vec<f64>,
    /// Geometric liquidity between token pairs
    liquidity_by_pair: PairwiseLiquidities,
}

impl Router {
    pub fn new(pools: Vec<UniV2Pool>) -> Self {
        let token_index = pools
            .iter()
            .flat_map(|p| [p.token0, p.token1])
            .unique()
            .enumerate()
            .map(|(i, token)| (token, i))
            .collect::<HashMap<_, _>>();

        let n_tokens = token_index.len();

        let mut reserve_by_token = vec![0.0; n_tokens];
        let mut liquidity_by_pair = PairwiseLiquidities::with_size(n_tokens);

        for pool in &pools {
            let index_0 = token_index[pool.token0];
            let index_1 = token_index[pool.token1];
            reserve_by_token[index_0] += pool.reserve0;
            reserve_by_token[index_1] += pool.reserve1;
            let liquidity = (pool.reserve0 * pool.reserve1).sqrt();
            *liquidity_by_pair.get_mut(index_0, index_1) += liquidity;
        }

        Router {
            n_tokens,
            token_index,
            reserve_by_token,
            liquidity_by_pair,
            q_by_token: vec![1.0; n_tokens],
        }
    }

    /// Solves for the maximum output amount of `output_token` that can be obtained by selling
    /// `input_amount` of `input_token`, updating the internal state of the router accordingly.
    ///
    /// This is done by adjusting the total reserves of `input_token`, then computing the
    /// no-arbitrage equilibrium to find out how much `output_token` can be extracted.
    pub fn solve(&mut self, input_token: &str, output_token: &str, input_amount: f64) -> f64 {
        let input_token = self.token_index[input_token];
        let output_token = self.token_index[output_token];

        self.reserve_by_token[input_token] += input_amount;
        let output_amount = self.no_arbitrage_equilibrium(output_token);

        output_amount
    }

    /// Iteratively solves for the no-arbitrage equilibrium using fixed-point iteration, computing
    /// the maximum output amount of output_token `f` that can be extracted in the process.
    ///
    /// The system enforces the conservation of total token balances:
    ///
    /// ```text
    ///   ∑ K_{u v} * (q_u / q_v) = T_u,   for all u ≠ output_token
    /// ```
    ///
    /// where:
    /// - `K_{u v} = Σ √kᵢ`  is the geometric liquidity between `u` and `v` (sum over all pools i
    ///   containing both `u` and `v`)
    /// - `T_u` is the total amount of token `u` across all pools
    /// - `q_u` is the sqrt-price variable for token `u`
    ///
    /// The fixed-point iteration updates until convergence (`max_relative_change < TOLERANCE`):
    ///
    /// ```text
    ///   q_u ← T_u / ( ∑ K_{u v} / q_v ),   for all u ≠ output_token
    /// ```
    ///
    /// Once the prices `q` have converged, the post-equilibrium total of the output token is:
    ///
    /// ```text
    ///   T'_f = ∑ K_{f v} * (q_f / q_v),
    /// ```
    ///
    /// and the amount of output token that can be extracted is:
    ///
    /// ```text
    ///   Δf = T_f − T'_f.
    /// ```
    ///
    /// Complexity:  `O(MAX_ITERS × n^2)`, where n is the number of tokens.
    fn no_arbitrage_equilibrium(&mut self, output_token: usize) -> f64 {
        for _ in 0..MAX_ITERS {
            let mut max_relative_change = 0.0;

            for token in 0..self.n_tokens {
                // Skip the output token; its price is not updated
                if token == output_token {
                    continue;
                }
                let q = self.q_by_token[token];

                // Update q_u ← T_u / ( ∑ K_{u v} / q_v )
                let mut denom = 0.0;
                for paired_token in 0..self.n_tokens {
                    let k_tv = self.liquidity_by_pair.get(token, paired_token);
                    denom += k_tv / self.q_by_token[paired_token];
                }
                let updated_q = self.reserve_by_token[token] / denom;

                let relative_change = ((updated_q - q).abs()) / q;

                if relative_change > max_relative_change {
                    max_relative_change = relative_change;
                }

                self.q_by_token[token] = updated_q;
            }

            if max_relative_change < TOLERANCE {
                break;
            }
        }

        // Optional renormalization: keep first token as price reference
        self.normalize_prices();

        // Compute and update the output reserve based after equilibrium
        let mut output_reserve = 0.0;
        for token in 0..self.n_tokens {
            let k_bv = self.liquidity_by_pair.get(output_token, token);
            output_reserve += k_bv * (self.q_by_token[output_token] / self.q_by_token[token]);
        }
        let extracted_amount = self.reserve_by_token[output_token] - output_reserve;
        self.reserve_by_token[output_token] = output_reserve;

        extracted_amount
    }

    fn normalize_prices(&mut self) {
        const REFERENCE_TOKEN: usize = 0;
        let ref_price = self.q_by_token[REFERENCE_TOKEN];
        for price in &mut self.q_by_token {
            *price /= ref_price;
        }
    }
}
