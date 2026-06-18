//! Step 3: build the default-extraction PipelineSpec from a session config.
//! Behind the `pipeline` feature so the legacy path stays default until this
//! matches the golden. Step 5 generalizes this into full spec selection
//! (choosing CommitRangeSource when --git-diff-shas is set, etc.).

use anyhow::Result;
use gnaw_core::configuration::GnawConfig;
use gnaw_core::path::{display_name, traverse_directory};
use gnaw_core::pipeline::adapters::{
    HandlebarsRenderer, IdentityChunker, PassThrough, RendererConfig, TakeUntilBudget,
    TiktokenCounter, Uniform, WorkingTreeSource,
};
use gnaw_core::pipeline::{PipelineSpec, Rendered, SourceOpts, run};
/// Build and run the default-extraction pipeline. Returns the rendered output.
///
/// NOTE the tree problem below — the renderer needs `source_tree` and
/// `absolute_code_path`, which only `traverse_directory` produces today. For
/// step 3 we get them by running the legacy traversal JUST for the tree, then
/// the pipeline produces the file contents. That double-walk is temporary and
/// ugly; step 4's `load_changed_tree` and a tree-from-items approach replace
/// it. Flagged loudly because it's the seam most likely to break parity.
pub fn run_default_extraction(config: &GnawConfig) -> Result<Rendered> {
    // Tree + path: legacy traversal, tree only. Its file list is discarded —
    // the pipeline source produces the actual file contents.
    let traversal = traverse_directory(config, None)?;
    let source_tree = traversal.tree;
    let absolute_code_path = display_name(&config.path);

    let renderer = HandlebarsRenderer::new(RendererConfig {
        absolute_code_path,
        source_tree,
        no_codeblock: config.no_codeblock,
        line_numbers: config.line_numbers,
        git_diff: None,
        template_str: config.template_str.clone(),
        template_name: config.template_name.clone(),
        output_format: config.output_format,
        user_variables: config.user_variables.clone(),
    });

    let spec = PipelineSpec {
        source: Box::new(WorkingTreeSource::new(config.clone())),
        selector: Box::new(PassThrough),
        chunker: Box::new(IdentityChunker),
        ranker: Box::new(Uniform),
        budgeter: Box::new(TakeUntilBudget::new(Box::new(TiktokenCounter::new(
            config.encoding,
        )))),
        renderer: Box::new(renderer),
        budget: 0,
    };

    let rendered = run(&spec, &SourceOpts::default())?;
    Ok(rendered)
}
