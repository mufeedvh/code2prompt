//! This module encapsulates the logic for counting the tokens in the rendered text.

use colored::*;
use tiktoken_rs::{cl100k_base, p50k_base, p50k_edit, r50k_base, CoreBPE};

/// Returns the appropriate tokenizer based on the provided encoding.
///
/// # Arguments
///
/// * `encoding` - An optional string specifying the encoding to use for tokenization.
///                Supported encodings: "cl100k" (default), "p50k", "p50k_edit", "r50k", "gpt2".
///
/// # Returns
///
/// * `CoreBPE` - The tokenizer corresponding to the specified encoding.
pub fn get_tokenizer(encoding: &Option<String>) -> CoreBPE {
    match encoding.as_deref().unwrap_or("cl100k") {
        "cl100k" => cl100k_base().unwrap(),
        "p50k" => p50k_base().unwrap(),
        "p50k_edit" => p50k_edit().unwrap(),
        "r50k" | "gpt2" => r50k_base().unwrap(),
        _ => cl100k_base().unwrap(),
    }
}

/// Returns the model information based on the provided encoding.
///
/// # Arguments
///
/// * `encoding` - An optional string specifying the encoding to use for retrieving model information.
///                Supported encodings: "cl100k" (default), "p50k", "p50k_edit", "r50k", "gpt2".
///
/// # Returns
///
/// * `&'static str` - A string describing the models associated with the specified encoding.
pub fn get_model_info(encoding: &Option<String>) -> &'static str {
    match encoding.as_deref().unwrap_or("cl100k") {
        "cl100k" => "ChatGPT models, text-embedding-ada-002",
        "p50k" => "Code models, text-davinci-002, text-davinci-003",
        "p50k_edit" => "Edit models like text-davinci-edit-001, code-davinci-edit-001",
        "r50k" | "gpt2" => "GPT-3 models like davinci",
        _ => "ChatGPT models, text-embedding-ada-002",
    }
}

/// Counts the tokens in the rendered text using the specified encoding and prints the result.
///
/// # Arguments
///
/// * `rendered` - The rendered template string.
/// * `encoding` - An optional string specifying the encoding to use for token counting.
///                Supported encodings: "cl100k" (default), "p50k", "p50k_edit", "r50k", "gpt2".
pub fn count_tokens(rendered: &str, encoding: &Option<String>) {
    let (bpe, model_info) = match encoding.as_deref().unwrap_or("cl100k") {
        "cl100k" => (cl100k_base(), "ChatGPT models, text-embedding-ada-002"),
        "p50k" => (
            p50k_base(),
            "Code models, text-davinci-002, text-davinci-003",
        ),
        "p50k_edit" => (
            p50k_edit(),
            "Edit models like text-davinci-edit-001, code-davinci-edit-001",
        ),
        "r50k" | "gpt2" => (r50k_base(), "GPT-3 models like davinci"),
        _ => (cl100k_base(), "ChatGPT models, text-embedding-ada-002"),
    };

    let token_count = bpe.unwrap().encode_with_special_tokens(rendered).len();

    println!(
        "{}{}{} Token count: {}, Model info: {}",
        "[".bold().white(),
        "i".bold().blue(),
        "]".bold().white(),
        token_count.to_string().bold().yellow(),
        model_info
    );
}
