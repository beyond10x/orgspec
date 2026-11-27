//! The orgspec checker.
//!
//! - [`check::check_design`] holds an organization design (`orgspec-organization/1`) to the
//!   structural rules of every design and to the rules of the organization types it names.
//! - [`check::check_type`] holds an organization type (`orgspec-type/1`) to the types it extends.
//!
//! The formats and the rules are described in `design/building-blocks.md` and
//! `design/organization-types.md`. The blocks and their fields are data in [`blocks`].

pub mod blocks;
pub mod check;
pub mod design;
pub mod graph;
pub mod node;
pub mod rules;
pub mod structure;
pub mod types;
pub mod typeset;
