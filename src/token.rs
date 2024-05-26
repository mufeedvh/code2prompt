use colored::*;
use tiktoken_rs::{cl100k_base, p50k_base, p50k_edit, r50k_base};

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
