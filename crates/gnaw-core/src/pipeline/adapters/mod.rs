//! Step 2: concrete adapters wrapping existing behavior. Each delegates to a
//! function that already works — adapting, not rewriting. Identity/no-op
//! impls for chunk/select/rank/budget; the trait having an impl is the win,
//! a clever impl is a separate feature.

//! Step 2: concrete adapters wrapping existing behavior.

mod budgeter;
mod chunker;
mod counter;
mod ranker;
mod renderer;
mod selector;
mod source;

pub use budgeter::TakeUntilBudget;
pub use chunker::IdentityChunker;
pub use counter::TiktokenCounter;
pub use ranker::Uniform;
pub use renderer::{HandlebarsRenderer, RendererConfig};
pub use selector::PassThrough;
pub use source::{CommitRangeSource, WorkingTreeSource};
