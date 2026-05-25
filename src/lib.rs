//! Native engine for llm-project-mapper.
//!
//! Faithful Rust port of the project-mapping pass that previously lived in
//! `bin/auto-map.js`. It inspects a repository locally and infers stack,
//! commands, service URLs, domain, entities, features, integrations and
//! directory shape, then renders the project map.

pub mod detect;
pub mod hamt;
pub mod profile;
pub mod render;
pub mod scan;
pub mod text;
pub mod virality;
pub mod yool;

pub use profile::{build_profile, Commands, Feature, Profile};
