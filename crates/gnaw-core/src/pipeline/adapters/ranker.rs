// crates/gnaw-core/src/pipeline/adapters/ranker.rs
//! No-op ranker: every chunk scores equal, so the budgeter sees source order.
//! A relevance/embedding ranker is a separate, later feature.

use crate::pipeline::{Chunk, RankCtx, Ranker};

pub struct Uniform;

impl Ranker for Uniform {
    fn score(&self, _chunk: &Chunk, _ctx: &RankCtx) -> f32 {
        0.0
    }
}
