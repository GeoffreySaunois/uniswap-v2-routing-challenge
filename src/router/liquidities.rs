#[derive(Debug)]
pub(super) struct PairwiseLiquidities {
    inner: Vec<Vec<f64>>,
}

impl PairwiseLiquidities {
    pub(super) fn with_size(num_tokens: usize) -> Self {
        Self {
            inner: vec![vec![0.0; num_tokens]; num_tokens],
        }
    }

    pub fn get(&self, token_a: usize, token_b: usize) -> f64 {
        let (a, b) = self.index(token_a, token_b);
        self.inner[a][b]
    }

    pub fn get_mut(&mut self, token_a: usize, token_b: usize) -> &mut f64 {
        let (a, b) = self.index(token_a, token_b);
        &mut self.inner[a][b]
    }

    fn index(&self, a: usize, b: usize) -> (usize, usize) {
        if a < b { (a, b) } else { (b, a) }
    }
}
