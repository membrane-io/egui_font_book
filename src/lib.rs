//! # egui-font-book
//!
//! An egui widget for browsing, downloading, and previewing fonts from curated
//! online sources (Google Fonts). Useful for choosing the right font for your
//! application.
//!
//! ## Quick start
//!
//! ```no_run
//! use egui_font_book::{FontBookState, FontBrowser};
//!
//! let mut state = FontBookState::new();
//!
//! // Inside your egui update loop:
//! // FontBrowser::new(&mut state).show(ui);
//! ```

mod browser;
mod catalog;
mod state;

pub use browser::FontBrowser;
pub use catalog::{FontCategory, FontEntry, FontVariant};
pub use state::{FontBookState, FontEntryBuilder, FontPreset, SavedState};
