//! This module encapsulates the logic for counting the tokens in the rendered text.
use std::str::FromStr;
use tiktoken_rs::{cl100k_base, o200k_base, p50k_base, p50k_edit, r50k_base};

#[derive(Debug, Clone)]
pub enum TokenFormat {
    Raw,
    Format,
}

/// Parses a string into a [`TokenFormat`].
impl FromStr for TokenFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "raw" => Ok(TokenFormat::Raw),
            "format" => Ok(TokenFormat::Format),
            _ => Err(format!(
                "Invalid token format: {}. Use 'raw' or 'format'.",
                s
            )),
        }
    }
}

/// Tokenizer types supported by tiktoken.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenizerType {
    O200kBase,
    Cl100kBase,
    P50kBase,
    P50kEdit,
    R50kBase,
    Gpt2,
}

/// Returns a description of the tokenizer type.
impl TokenizerType {
    pub fn description(&self) -> &'static str {
        match self {
            TokenizerType::O200kBase => "OpenAI models, ChatGPT-4o",
            TokenizerType::Cl100kBase => "ChatGPT models, text-embedding-ada-002",
            TokenizerType::P50kBase => "Code models, text-davinci-002, text-davinci-003",
            TokenizerType::P50kEdit => {
                "Edit models like text-davinci-edit-001, code-davinci-edit-001"
            }
            TokenizerType::R50kBase => "GPT-3 models like davinci",
            TokenizerType::Gpt2 => "GPT-2 tokenizer",
        }
    }
}

/// Parses a string into a [`TokenizerType`].
impl FromStr for TokenizerType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "o200k" => Ok(TokenizerType::O200kBase),
            "cl100k" => Ok(TokenizerType::Cl100kBase),
            "p50k" => Ok(TokenizerType::P50kBase),
            "p50k_edit" => Ok(TokenizerType::P50kEdit),
            "r50k" | "gpt2" => Ok(TokenizerType::R50kBase),
            _ => Err(()),
        }
    }
}

/// Counts the tokens in the rendered text using the specified encoding and prints the result.
///
/// # Arguments
///
/// * `rendered` - The rendered template string.
/// * `tokenizer_type` - The tokenizer type to use.
pub fn count_tokens(rendered: &str, tokenizer_type: &TokenizerType) -> usize {
    let bpe = match tokenizer_type {
        TokenizerType::O200kBase => o200k_base(),
        TokenizerType::Cl100kBase => cl100k_base(),
        TokenizerType::P50kBase => p50k_base(),
        TokenizerType::P50kEdit => p50k_edit(),
        TokenizerType::R50kBase | TokenizerType::Gpt2 => r50k_base(),
    };

    let token_count = bpe.unwrap().encode_with_special_tokens(rendered).len();
    token_count
}
