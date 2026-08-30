//! Deliberately optimizer-free compiler path.
//!
//! This module contains only the direct translation pipeline.  It deliberately
//! has no dependency on `optimized::optimizer`.
pub mod pipeline;
