//! Renderer wrapping handlebars + the existing default templates. Owns
//! PRESENTATION — fences (via the template's no_codeblock branch) and line
//! numbers (applied here, since the source no longer wraps). `Selection`
//! stays clean: top-level context (tree, path, no_codeblock flag) lives on
//! this adapter's constructor, NOT on the wire DTO.
//!
//! Maps `Selection.chunks` onto the `{{#each files}}` shape the legacy
//! templates expect, so step 3 can diff against `default.golden.md` byte-for-
//! byte without changing the templates. This is option-3 from the step-2
//! renderer decision: legacy templates fed from the new pipeline shape.

use crate::path::wrap_code_block;
use crate::pipeline::{PipelineError, RenderContext, Rendered, Renderer, Selection};
use crate::template::{OutputFormat, handlebars_setup, render_template};
use serde::Serialize;
use std::collections::HashMap;

/// Everything the renderer needs that isn't in `Selection`. Constructed by
/// the frontend (CLI today, a REST handler later) from config + loaded
/// context. Keeping this a struct rather than a wide constructor is also what
/// a REST handler wants — it builds one of these from a request body.
pub struct RendererConfig {
    /// Presentation context the pipeline shape doesn't carry.
    pub no_codeblock: bool,
    pub line_numbers: bool,
    pub git_diff: Option<String>,
    pub user_variables: HashMap<String, String>,
    /// Template body + name. Empty body → fall back to the format default.
    pub template_str: String,
    pub template_name: String,
    pub output_format: OutputFormat,
}

/// One file as the legacy templates expect it. Mirrors the fields
/// `default_template_md.hbs` / `_xml.hbs` read off each `files` entry.
/// Deliberately NOT `FileEntry` — we only supply what the templates touch.
#[derive(Serialize)]
struct RenderFile {
    path: String,
    extension: String,
    code: String,
}

/// Top-level render context. Flattens user variables the way the legacy
/// `TemplateContext` did, so a custom template referencing `{{project}}`
/// still resolves.
#[derive(Serialize)]
struct RenderContextHbs<'a> {
    absolute_code_path: &'a str,
    source_tree: &'a str,
    files: Vec<RenderFile>,
    #[serde(skip_serializing_if = "Option::is_none")]
    git_diff: &'a Option<String>,
    no_codeblock: bool,
    #[serde(flatten)]
    user_variables: &'a HashMap<String, String>,
}

pub struct HandlebarsRenderer {
    cfg: RendererConfig,
}

impl HandlebarsRenderer {
    pub fn new(cfg: RendererConfig) -> Self {
        Self { cfg }
    }

    /// Resolve template body + name, mirroring session.rs's selection: an
    /// empty configured body falls back to the format's built-in default.
    fn resolve_template(&self) -> (String, String) {
        if self.cfg.template_str.is_empty() {
            let body = match self.cfg.output_format {
                OutputFormat::Markdown => include_str!("../../default_template_md.hbs").to_string(),
                OutputFormat::Xml | OutputFormat::Json => {
                    include_str!("../../default_template_xml.hbs").to_string()
                }
            };
            let name = match self.cfg.output_format {
                OutputFormat::Markdown => "markdown".to_string(),
                OutputFormat::Xml | OutputFormat::Json => "xml".to_string(),
            };
            (body, name)
        } else {
            (
                self.cfg.template_str.clone(),
                self.cfg.template_name.clone(),
            )
        }
    }
}

impl Renderer for HandlebarsRenderer {
    fn render(&self, sel: &Selection, ctx: &RenderContext) -> Result<Rendered, PipelineError> {
        // Map chunks → the files array. Line numbers are applied HERE because
        // the source no longer wraps; fences stay the template's job via
        // no_codeblock. This is the presentation ownership option 3 buys.
        let files: Vec<RenderFile> = sel
            .chunks
            .iter()
            .map(|c| RenderFile {
                path: c.source_path.clone(),
                extension: c.extension.clone(),
                code: wrap_code_block(&c.text, self.cfg.line_numbers),
            })
            .collect();

        let render_ctx = RenderContextHbs {
            absolute_code_path: &ctx.absolute_code_path,
            source_tree: &ctx.source_tree,
            files,
            git_diff: &self.cfg.git_diff,
            no_codeblock: self.cfg.no_codeblock,
            user_variables: &self.cfg.user_variables,
        };

        let (body_template, name) = self.resolve_template();
        let hb = handlebars_setup(&body_template, &name)
            .map_err(|e| PipelineError::Render(e.to_string()))?;
        let body = render_template(&hb, &name, &render_ctx)
            .map_err(|e| PipelineError::Render(e.to_string()))?;

        let format = match self.cfg.output_format {
            OutputFormat::Markdown => "markdown",
            OutputFormat::Xml => "xml",
            OutputFormat::Json => "json",
        }
        .to_string();

        Ok(Rendered {
            body,
            format,
            tally: sel.tally.clone(),
        })
    }
}
