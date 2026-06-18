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
    ItemsTree, PassThrough, PatternSelector, RendererConfig, TakeUntilBudget, TiktokenCounter,
    Uniform, WorkingTreeSource,
};
use gnaw_core::pipeline::ports::TreeBuilder;
use gnaw_core::pipeline::{PipelineSpec, Rendered, SourceOpts, run};
/// Build and run the default-extraction pipeline. Returns the rendered output.
///
/// Build and run the default-extraction pipeline. Returns the rendered output.
///
/// NOTE the tree problem below — the renderer needs `source_tree` and
/// `absolute_code_path`, which only `traverse_directory` produces today. For
/// step 3 we get them by running the legacy traversal JUST for the tree, then
/// the pipeline produces the file contents. That double-walk is temporary and
/// ugly; step 4's `load_changed_tree` and a tree-from-items approach replace
/// it. Flagged loudly because it's the seam most likely to break parity.
pub fn run_default_extraction(config: &GnawConfig) -> Result<Rendered> {
    let source_tree_override = if config.full_directory_tree {
        // Full-directory-tree means "show every directory, ignore include/exclude
        // for the tree" — so it can't come from the filtered items. Walk for the
        // tree; the pipeline still produces the actual file contents.
        let traversal = gnaw_core::path::traverse_directory(config, None)?;
        Some(traversal.tree)
    } else {
        None
    };
    // Tree + path: legacy traversal, tree only. Its file list is discarded —
    // the pipeline source produces the actual file contents.
    let renderer = HandlebarsRenderer::new(RendererConfig {
        no_codeblock: config.no_codeblock,
        line_numbers: config.line_numbers,
        git_diff: None,
        template_str: config.template_str.clone(),
        template_name: config.template_name.clone(),
        output_format: config.output_format,
        user_variables: config.user_variables.clone(),
    });

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

    // The changed-files template is structural, not a format default — the
    // builder selects it. An explicit --template still wins: if the user set
    // one, config.template_str is non-empty and we honor it.
    let (template_str, template_name) = if config.template_str.is_empty() {
        let t = BuiltinTemplates::get_template("git-diff-shas-pipeline")
            .ok_or_else(|| anyhow::anyhow!("builtin git-diff-shas-pipeline template missing"))?;
        (t.content.to_string(), "git-diff-shas-pipeline".to_string())
    } else {
        (config.template_str.clone(), config.template_name.clone())
    };

    let renderer = HandlebarsRenderer::new(RendererConfig {
        no_codeblock: config.no_codeblock,
        line_numbers: config.line_numbers,
        git_diff: None,
        template_str,
        template_name,
        output_format: config.output_format,
        user_variables: config.user_variables.clone(),
    });

    let spec = PipelineSpec {
        source: Box::new(CommitRangeSource::new(config.clone(), ref1, ref2)),
        selector: Box::new(PatternSelector::new(
            &config.include_patterns,
            &config.exclude_patterns,
        )),
        chunker: Box::new(ChangedChunker),
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
