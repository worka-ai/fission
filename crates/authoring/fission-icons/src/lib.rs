//! Compile-time Material Design icon access for the Fission UI framework.
//!
//! Icons are organized as `material::<category>::<icon_name>::<variant>()` and
//! return `&'static str` containing the 24px SVG markup.
//!
//! # Usage
//!
//! ```rust,ignore
//! use fission_icons::material;
//! let svg: &str = material::navigation::close::regular();
//! ```
//!
//! # Variants
//!
//! Each icon may have up to five style variants: `regular()`, `outlined()`,
//! `round()`, `sharp()`, and `two_tone()`.

/// Material Design icon set, generated at build time from SVG source files.
///
/// Access icons via `material::<category>::<icon_name>::<variant>()`.
/// For example: `material::action::home::regular()` returns the home icon SVG.
///
/// When the `reflection` feature is enabled, call `all_icons()` to get a
/// `Vec` of `(category, name, variant, fn() -> &'static str)` tuples.
pub mod material {
    include!(concat!(env!("OUT_DIR"), "/material_icons.rs"));
}
