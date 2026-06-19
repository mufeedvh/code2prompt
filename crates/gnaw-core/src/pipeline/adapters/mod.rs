//! Step 2: concrete adapters wrapping existing behavior. Each delegates to a
//! function that already works — adapting, not rewriting. Identity/no-op
//! impls for chunk/select/rank/budget; the trait having an impl is the win,
//! a clever impl is a separate feature.

//! Step 2: concrete adapters wrapping existing behavior.

mod budgeter;
mod changed_chunker;
mod chunker;
mod counter;
mod ranker;
mod renderer;
mod scrubber;
mod selector;
mod source;
mod tree;

pub use budgeter::TakeUntilBudget;
pub use changed_chunker::ChangedChunker;
pub use chunker::IdentityChunker;
pub use counter::TiktokenCounter;
pub use ranker::Uniform;
pub use renderer::{HandlebarsRenderer, RendererConfig};
pub use scrubber::SecretScrubber;
pub use selector::{PassThrough, PatternSelector};
pub use source::{CommitRangeSource, WorkingTreeSource};
pub use tree::{FullWalkTree, ItemsTree};
