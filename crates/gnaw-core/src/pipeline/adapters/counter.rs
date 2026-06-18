//! Token counter wrapping the existing tiktoken path. The ONE place counting
//! happens in the new architecture; counts raw chunk text, so the tally
//! measures code, not formatting.

use crate::pipeline::TokenCounter;
use crate::tokenizer::{TokenizerType, count_tokens};

pub struct TiktokenCounter {
    encoding: TokenizerType,
}

impl TiktokenCounter {
    pub fn new(encoding: TokenizerType) -> Self {
        Self { encoding }
    }
}

impl TokenCounter for TiktokenCounter {
    fn count(&self, text: &str) -> usize {
        count_tokens(text, &self.encoding)
    }

    fn encoding(&self) -> &str {
        match self.encoding {
            TokenizerType::O200kBase => "o200k_base",
            TokenizerType::Cl100kBase => "cl100k_base",
            TokenizerType::P50kBase => "p50k_base",
            TokenizerType::P50kEdit => "p50k_edit",
            TokenizerType::R50kBase => "r50k_base",
        }
    }
}
