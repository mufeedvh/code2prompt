//! Step 3: build the default-extraction PipelineSpec from a session config.
//! Behind the `pipeline` feature so the legacy path stays default until this
//! matches the golden. Step 5 generalizes this into full spec selection
//! (choosing CommitRangeSource when --git-diff-shas is set, etc.).

use anyhow::Result;
use gnaw_core::builtin_templates::BuiltinTemplates;
use gnaw_core::configuration::GnawConfig;
use gnaw_core::path::display_name;
use gnaw_core::pipeline::adapters::{
    ChangedChunker, CommitRangeSource, FullWalkTree, HandlebarsRenderer, IdentityChunker,
    ItemsTree, PatternSelector, RendererConfig, SecretScrubber, TakeUntilBudget, TiktokenCounter,
    Uniform, WorkingTreeSource,
};
use gnaw_core::pipeline::ports::TreeBuilder;
use gnaw_core::pipeline::{PipelineSpec, Rendered, SourceOpts, run};
/// Build and run the default (working-tree) extraction pipeline, returning the
/// rendered output.
///
/// Source is `WorkingTreeSource`; the source tree is derived from the surviving
/// items by `ItemsTree` (or a full filesystem walk via `FullWalkTree` when
/// `--full-directory-tree` is set), so there is no separate walk just for the
/// tree — the earlier double-walk is gone. Step 5 folds this and
/// `run_changed_files_extraction` into a single `build_spec(config)` that picks
/// the source from config rather than the caller picking the function.
pub fn run_default_extraction(config: &GnawConfig) -> Result<Rendered> {
    // Tree + path: legacy traversal, tree only. Its file list is discarded —
    // the pipeline source produces the actual file contents.
    let renderer = HandlebarsRenderer::new(default_renderer_config(config));

    let tree_builder: Box<dyn TreeBuilder> = if config.full_directory_tree {
        Box::new(FullWalkTree::new(config.clone()))
    } else {
        Box::new(ItemsTree)
    };

    let spec = PipelineSpec {
        source: Box::new(WorkingTreeSource::new(config.clone())),
        selector: Box::new(PatternSelector::new(
            &config.include_patterns,
            &config.exclude_patterns,
        )),
        chunker: Box::new(IdentityChunker),
        scrubber: Box::new(SecretScrubber::new(config)),
        ranker: Box::new(Uniform),
        budgeter: Box::new(TakeUntilBudget::new(Box::new(TiktokenCounter::new(
            config.encoding,
        )))),
        renderer: Box::new(renderer),
        tree_builder,
        budget: 0,
        root_label: display_name(&config.path),
        sort_method: config.sort_method,
    };

    let rendered = run(&spec, &SourceOpts)?;
    Ok(rendered)
}

/// Step 4: the changed-files extraction spec. Selected when --git-diff-shas is
/// set. Uses CommitRangeSource (no working-tree walk — the whole point) +
/// ChangedChunker (formats per-file blocks, keeps binaries) + the pipeline
/// changed-files template. The tree is built by `run` from the changed items,
/// so it lists only touched files.
pub fn run_changed_files_extraction(config: &GnawConfig) -> Result<Rendered> {
    let (ref1, ref2) = config
        .diff_shas
        .clone()
        .ok_or_else(|| anyhow::anyhow!("run_changed_files_extraction called without diff_shas"))?;

    let renderer = HandlebarsRenderer::new(changed_renderer_config(config)?);

    let spec = PipelineSpec {
        source: Box::new(CommitRangeSource::new(config.clone(), ref1, ref2)),
        selector: Box::new(PatternSelector::new(
            &config.include_patterns,
            &config.exclude_patterns,
        )),
        chunker: Box::new(ChangedChunker),
        scrubber: Box::new(SecretScrubber::new(config)),
        ranker: Box::new(Uniform),
        budgeter: Box::new(TakeUntilBudget::new(Box::new(TiktokenCounter::new(
            config.encoding,
        )))),
        renderer: Box::new(renderer),
        tree_builder: Box::new(ItemsTree),
        budget: 0,
        root_label: display_name(&config.path),
        sort_method: config.sort_method,
    };

    let rendered = run(&spec, &SourceOpts)?;
    Ok(rendered)
}

fn default_renderer_config(config: &GnawConfig) -> RendererConfig {
    RendererConfig {
        no_codeblock: config.no_codeblock,
        line_numbers: config.line_numbers,
        git_diff: None,
        template_str: config.template_str.clone(),
        template_name: config.template_name.clone(),
        output_format: config.output_format,
        user_variables: config.user_variables.clone(),
    }
}

/// Renderer config for the changed-files view: resolves the built-in
/// git-diff-shas-pipeline template unless the user set an explicit one.
fn changed_renderer_config(config: &GnawConfig) -> Result<RendererConfig> {
    let (template_str, template_name) = if config.template_str.is_empty() {
        let t = BuiltinTemplates::get_template("git-diff-shas-pipeline")
            .ok_or_else(|| anyhow::anyhow!("builtin git-diff-shas-pipeline template missing"))?;
        (t.content.to_string(), "git-diff-shas-pipeline".to_string())
    } else {
        (config.template_str.clone(), config.template_name.clone())
    };
    Ok(RendererConfig {
        no_codeblock: config.no_codeblock,
        line_numbers: config.line_numbers,
        git_diff: None,
        template_str,
        template_name,
        output_format: config.output_format,
        user_variables: config.user_variables.clone(),
    })
}

/// Build the renderer matching whichever extraction `config` selects, so a split
/// part renders byte-identically to a whole run. Used by the CLI split path,
/// which renders each part via `render_subset`.
pub fn build_renderer_for(config: &GnawConfig) -> Result<HandlebarsRenderer> {
    let cfg = if config.diff_shas.is_some() {
        changed_renderer_config(config)?
    } else {
        default_renderer_config(config)
    };
    Ok(HandlebarsRenderer::new(cfg))
}
