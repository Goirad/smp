pub struct StreamingVariance {
    count: u64,
    mean: f64,
    variance: f64,
}

impl StreamingVariance {
    pub fn new() -> Self {
        StreamingVariance {
            count: 0,
            mean: 0.0,
            variance: 0.0,
        }
    }

    pub fn update(&mut self, new_val: f64) {
        self.count += 1;
        let delta = new_val - self.mean;
        self.mean += delta / (self.count as f64);
        let delta2 = new_val - self.mean;
        self.variance += delta * delta2;
    }

    #[allow(unused)]
    pub fn mean(&self) -> f64 {
        self.mean
    }

    #[allow(unused)]
    pub fn variance(&self) -> f64 {
        self.variance / (self.count as f64)
    }

    #[allow(unused)]
    pub fn standard_deviation(&self) -> f64 {
        self.variance().sqrt()
    }

    #[allow(unused)]
    pub fn sample_variance(&self) -> f64 {
        self.variance / (self.count as f64 - 1.0)
    }

    #[allow(unused)]
    pub fn sample_standard_deviation(&self) -> f64 {
        self.sample_variance().sqrt()
    }

    #[allow(unused)]
    pub fn count(&self) -> u64 {
        self.count
    }
}
