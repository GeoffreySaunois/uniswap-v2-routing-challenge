#[derive(Debug, Clone)]
pub struct UniV2Pool {
    pub token0: &'static str,
    pub token1: &'static str,
    pub reserve0: f64,
    pub reserve1: f64,
}

impl UniV2Pool {
    pub fn new(token0: &'static str, token1: &'static str, reserve0: f64, reserve1: f64) -> Self {
        Self {
            token0,
            token1,
            reserve0,
            reserve1,
        }
    }

    // Returns how many output tokens will be returned if a given amount of input token are added to
    // the pool.
    #[allow(unused)]
    pub fn get_output_amount(&self, input_token: &str, input_amount: f64) -> f64 {
        self.require_owned_token(input_token);

        let (reserve_in, reserve_out) = match input_token == self.token0 {
            true => (self.reserve0, self.reserve1),
            false => (self.reserve1, self.reserve0),
        };

        (input_amount * reserve_out) / (reserve_in + input_amount)
    }

    // Returns the instataneous price. This is given mostly for information purpose.
    #[allow(unused)]
    pub fn get_spot_price(&self, input_token: &str) -> f64 {
        self.require_owned_token(input_token);

        match input_token == self.token0 {
            true => self.reserve0 / self.reserve1,
            false => self.reserve1 / self.reserve0,
        }
    }

    #[allow(unused)]
    fn require_owned_token(&self, token: &str) {
        let is_owned = token == self.token0 || token == self.token1;

        if !is_owned {
            panic!("unsupported token");
        }
    }
}
