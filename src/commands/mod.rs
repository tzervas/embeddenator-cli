//! Command implementations for CLI operations

pub mod bundle_hier;
pub mod extract;
pub mod ingest;
pub mod mount;
pub mod query;
pub mod update;

pub use bundle_hier::handle_bundle_hier;
pub use extract::handle_extract;
pub use ingest::handle_ingest;
#[cfg(feature = "fuse")]
pub use mount::handle_mount;
pub use query::{handle_query, handle_query_text};
pub use update::{
    handle_update_add, handle_update_compact, handle_update_modify, handle_update_remove,
};
