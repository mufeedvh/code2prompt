use anyhow::Result;
use handlebars::{Handlebars, no_escape};
use regex::Regex;

/// Set up the Handlebars template engine with a default template string
pub fn handlebars_setup(template_str: &str) -> Result<Handlebars<'static>> {
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(no_escape);
    handlebars
        .register_template_string("default", template_str)
        .expect("Failed to register default template");

    Ok(handlebars)
}


/// Extracts the undefined variables from the template string
pub fn extract_undefined_variables(template: &str) -> Vec<String> {
    let registered_identifiers = vec!["path", "code", "git_diff"];
    let re = Regex::new(r"\{\{\s*(?P<var>[a-zA-Z_][a-zA-Z_0-9]*)\s*\}\}").unwrap();
    re.captures_iter(template)
        .map(|cap| cap["var"].to_string())
        .filter(|var| !registered_identifiers.contains(&var.as_str()))
        .collect()
}

/// Renders the template with the provided data
pub fn render_template(handlebars: &Handlebars, template_name: &str, data: &serde_json::Value) -> String {
    handlebars
        .render(template_name, data)
        .expect("Failed to render template")
        .trim()
        .to_string()
}
