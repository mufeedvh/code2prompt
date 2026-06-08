//! This module contains the functions to set up the Handlebars template engine and render the template with the provided data.
//! It also includes functions for handling user-defined variables, copying the rendered output to the clipboard, and writing it to a file.
use anyhow::{Result, anyhow};
use handlebars::template::{Parameter, Template, TemplateElement};
use handlebars::{Handlebars, no_escape};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::io::Write;

/// Set up the Handlebars template engine with a template string and a template name.
///
/// # Arguments
///
/// * `template_str` - The Handlebars template string.
/// * `template_name` - The name of the template.
///
/// # Returns
///
/// * `Result<Handlebars<'static>>` - The configured Handlebars instance.
pub fn handlebars_setup(template_str: &str, template_name: &str) -> Result<Handlebars<'static>> {
    let mut handlebars = Handlebars::new();
    handlebars.register_escape_fn(no_escape);

    handlebars
        .register_template_string(template_name, template_str)
        .map_err(|e| anyhow!("Failed to register template: {}", e))?;

    Ok(handlebars)
}

/// Identifiers gnaw injects into the render context itself: top-level session
/// data plus the per-file fields exposed inside `{{#each files}}`. Anything a
/// template references that is NOT one of these is a variable the user must
/// supply, so we prompt for it.
const PROVIDED_IDENTIFIERS: &[&str] = &[
    // top-level context
    "absolute_code_path",
    "source_tree",
    "files",
    "git_diff",
    "git_diff_branch",
    "git_log_branch",
    "no_codeblock",
    // per-file (FileEntry) fields, in scope within {{#each files}}
    "path",
    "code",
    "extension",
    "token_count",
    "metadata",
    "mod_time",
];

/// Extracts the user-supplied variables a template references.
///
/// Walks Handlebars' parsed AST instead of regex-scanning the source, so block
/// keywords (`{{else}}`), helper names (`each`, `if`, `unless`), data refs
/// (`{{@index}}`), and context navigation (`{{../x}}`, `{{this}}`) are never
/// mistaken for variables. Names gnaw already provides are filtered out;
/// whatever remains is prompted for. De-duplicated in first-seen order.
pub fn extract_undefined_variables(template: &str) -> Vec<String> {
    // A malformed template yields nothing here; the render step reports the
    // syntax error with a far better message than we could.
    let Ok(compiled) = Template::compile(template) else {
        return Vec::new();
    };
    let mut found = Vec::new();
    let mut seen = HashSet::new();
    collect_variables(&compiled.elements, &mut found, &mut seen);
    found
}

fn collect_variables(
    elements: &[TemplateElement],
    out: &mut Vec<String>,
    seen: &mut HashSet<String>,
) {
    for el in elements {
        match el {
            // {{ var }} and {{{ var }}}
            TemplateElement::Expression(ht) | TemplateElement::HtmlExpression(ht) => {
                // A bare interpolation's name is a Path (the variable). A helper
                // call's name is a Name (not a variable) — but its params may be.
                if matches!(ht.name, Parameter::Path(_)) {
                    push_variable(ht.name.as_name(), out, seen);
                }
                push_param_variables(&ht.params, out, seen);
            }
            // {{#each}} {{#if}} {{#unless}} {{#with}} {{#custom}} ...
            TemplateElement::HelperBlock(ht) => {
                // ht.name is the helper keyword — skip it. Iterated/tested paths
                // live in params; both block branches may reference variables.
                push_param_variables(&ht.params, out, seen);
                if let Some(body) = &ht.template {
                    collect_variables(&body.elements, out, seen);
                }
                if let Some(inverse) = &ht.inverse {
                    collect_variables(&inverse.elements, out, seen);
                }
            }
            // RawString, Comment, decorators, partials: no user variables.
            _ => {}
        }
    }
}

fn push_param_variables(params: &[Parameter], out: &mut Vec<String>, seen: &mut HashSet<String>) {
    for p in params {
        if matches!(p, Parameter::Path(_)) {
            push_variable(p.as_name(), out, seen);
        }
    }
}

fn push_variable(raw: Option<&str>, out: &mut Vec<String>, seen: &mut HashSet<String>) {
    let Some(raw) = raw else { return };
    let Some(head) = head_identifier(raw) else {
        return;
    };
    if PROVIDED_IDENTIFIERS.contains(&head) {
        return;
    }
    if seen.insert(head.to_owned()) {
        out.push(head.to_owned());
    }
}

/// Leading identifier of a Handlebars path, skipping context navigation
/// (`./`, `../`, `this`) and ignoring `@`-data (`@index`, `@root`, ...).
fn head_identifier(raw: &str) -> Option<&str> {
    for seg in raw.split(['/', '.']) {
        match seg {
            "" | "." | ".." | "this" => continue,
            s if s.starts_with('@') => return None,
            s => return Some(s),
        }
    }
    None
}

/// Renders the template with the provided data.
///
/// # Arguments
///
/// * `handlebars` - The configured Handlebars instance.
/// * `template_name` - The name of the template.
/// * `data` - Any serializable data object.
///
/// # Returns
///
/// * `Result<String>` - The rendered template as a string.
pub fn render_template<T: Serialize>(
    handlebars: &Handlebars,
    template_name: &str,
    data: &T,
) -> Result<String> {
    let rendered = handlebars
        .render(template_name, data)
        .map_err(|e| anyhow!("Failed to render template: {}", e))?;
    Ok(rendered.trim().to_string())
}

/// Writes the rendered template to a specified output file
///
/// # Arguments
///
/// * `output_path` - The path to the output file.
/// * `rendered` - The rendered template string.
///
/// # Returns
///
/// * `Result<()>` - An empty result indicating success or an error.
pub fn write_to_file(output_path: &str, rendered: &str) -> Result<()> {
    let file = std::fs::File::create(output_path)?;
    let mut writer = std::io::BufWriter::new(file);
    write!(writer, "{}", rendered)?;
    Ok(())
}

/// Enum to represent the output format.
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OutputFormat {
    #[default]
    Markdown,
    Json,
    Xml,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutputFormat::Markdown => write!(f, "markdown"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Xml => write!(f, "xml"),
        }
    }
}

#[cfg(test)]
mod undefined_variable_tests {
    use super::extract_undefined_variables as extract;

    #[test]
    fn plain_user_variable_is_found() {
        assert_eq!(extract("Hello {{name}}"), vec!["name"]);
    }

    #[test]
    fn else_keyword_is_never_a_variable() {
        assert_eq!(extract("{{#if x}}a{{else}}b{{/if}}"), vec!["x"]);
        assert_eq!(
            extract("{{#unless flag}}a{{else}}b{{/unless}}"),
            vec!["flag"]
        );
    }

    #[test]
    fn per_file_fields_are_provided_not_prompted() {
        assert!(extract("{{#each files}}{{path}} {{code}}{{/each}}").is_empty());
    }

    #[test]
    fn user_var_outside_each_is_found_while_fields_inside_are_not() {
        assert_eq!(
            extract("{{#each files}}{{path}}{{/each}}{{footer}}"),
            vec!["footer"]
        );
    }

    #[test]
    fn parent_and_data_refs_are_ignored() {
        assert!(extract("{{#each files}}{{../no_codeblock}}{{@index}}{{/each}}").is_empty());
    }

    #[test]
    fn triple_stache_is_handled() {
        assert_eq!(extract("{{{raw_thing}}}"), vec!["raw_thing"]);
    }

    #[test]
    fn duplicates_are_collapsed() {
        assert_eq!(extract("{{q}} ... {{q}}"), vec!["q"]);
    }

    #[test]
    fn provided_top_level_vars_are_not_prompted() {
        assert!(extract("{{absolute_code_path}} {{source_tree}} {{git_diff}}").is_empty());
    }
}
