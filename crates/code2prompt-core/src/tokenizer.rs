//! This module encapsulates the logic for counting the tokens in the rendered text.
use std::str::FromStr;
use tiktoken_rs::{cl100k_base, o200k_base, p50k_base, p50k_edit, r50k_base, CoreBPE};
use std::sync::OnceLock;

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

impl Default for TokenFormat {
    fn default() -> Self {
        TokenFormat::Raw
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

impl Default for TokenizerType {
    fn default() -> Self {
        TokenizerType::Cl100kBase
    }
}

// Cache tokenizers to avoid expensive re-initialization
static O200K_BASE: OnceLock<CoreBPE> = OnceLock::new();
static CL100K_BASE: OnceLock<CoreBPE> = OnceLock::new();
static P50K_BASE: OnceLock<CoreBPE> = OnceLock::new();
static P50K_EDIT: OnceLock<CoreBPE> = OnceLock::new();
static R50K_BASE: OnceLock<CoreBPE> = OnceLock::new();

/// Counts the tokens in the provided text using the specified tokenizer type.
///
/// # Arguments
///
/// * `rendered` - The text to count tokens in
/// * `tokenizer_type` - The tokenizer encoding to use
///
/// # Returns
///
/// * `usize` - The number of tokens in the text
pub fn count_tokens(rendered: &str, tokenizer_type: &TokenizerType) -> usize {
    use std::time::Instant;
    let start = Instant::now();
    
    let bpe = match tokenizer_type {
        TokenizerType::O200kBase => O200K_BASE.get_or_init(|| o200k_base().unwrap()),
        TokenizerType::Cl100kBase => CL100K_BASE.get_or_init(|| cl100k_base().unwrap()),
        TokenizerType::P50kBase => P50K_BASE.get_or_init(|| p50k_base().unwrap()),
        TokenizerType::P50kEdit => P50K_EDIT.get_or_init(|| p50k_edit().unwrap()),
        TokenizerType::R50kBase | TokenizerType::Gpt2 => R50K_BASE.get_or_init(|| r50k_base().unwrap()),
    };

    let token_count = bpe.encode_with_special_tokens(rendered).len();
    
    if std::env::var("DEBUG_TOKENIZER").is_ok() {
        eprintln!("Tokenized {} chars in {:?}", rendered.len(), start.elapsed());
    }
    
    token_count
}
