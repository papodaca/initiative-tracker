//! Campaign / combat domain shared by the GTK UI.
//!
//! Types mirror the Svelte runtime state with snake_case JSON going forward.
//! Behavioral helpers aim for parity with `Console.svelte` / `Presenter.svelte`.

mod combat;
mod state;
mod visibility;

pub use combat::*;
pub use state::*;
#[allow(unused_imports)] // used by Phase 3+ combat / presenter UI
pub use visibility::*;