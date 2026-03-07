//! Token budget enforcement.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenBudget {
    pub max_tokens: u64,
    pub used_tokens: u64,
}

impl TokenBudget {
    pub fn new(max_tokens: u64) -> Self {
        Self { max_tokens, used_tokens: 0 }
    }

    pub fn unlimited() -> Self {
        Self { max_tokens: u64::MAX, used_tokens: 0 }
    }

    pub fn remaining(&self) -> u64 {
        self.max_tokens.saturating_sub(self.used_tokens)
    }

    pub fn is_exhausted(&self) -> bool {
        self.used_tokens >= self.max_tokens
    }

    pub fn can_afford(&self, cost: u64) -> bool {
        self.remaining() >= cost
    }

    pub fn spend(&mut self, tokens: u64) -> bool {
        if self.can_afford(tokens) {
            self.used_tokens += tokens;
            true
        } else {
            false
        }
    }

    pub fn force_spend(&mut self, tokens: u64) {
        self.used_tokens += tokens;
    }

    pub fn utilization(&self) -> f64 {
        if self.max_tokens == 0 || self.max_tokens == u64::MAX { return 0.0; }
        self.used_tokens as f64 / self.max_tokens as f64
    }
}

impl Default for TokenBudget {
    fn default() -> Self {
        Self::unlimited()
    }
}
