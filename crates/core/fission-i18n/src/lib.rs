//! Internationalization (i18n) support for Fission applications.
//!
//! Provides a registry-based translation system that stores locale-specific
//! string bundles and retrieves messages by key and locale at runtime.
//!
//! # Example
//!
//! ```rust,ignore
//! use fission_i18n::{I18nRegistry, TranslationBundle, Locale};
//!
//! let mut registry = I18nRegistry::new();
//! registry.add_bundle(bundle);
//! let msg = registry.get(&Locale::from("en-US"), "greeting");
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A BCP 47 locale identifier (e.g., `"en-US"`, `"ja-JP"`).
///
/// Defaults to `"en-US"`. Implements `From<&str>` for convenient construction.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Locale(pub String);

impl Default for Locale {
    fn default() -> Self {
        Locale("en-US".to_string())
    }
}

impl From<&str> for Locale {
    fn from(s: &str) -> Self {
        Locale(s.to_string())
    }
}

/// A collection of key-value message pairs for a single locale.
///
/// Load from JSON, TOML, or construct programmatically. Add to an
/// [`I18nRegistry`] via [`add_bundle()`](I18nRegistry::add_bundle).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TranslationBundle {
    /// The locale this bundle provides translations for.
    pub locale: Locale,
    /// Map of message keys to translated strings.
    pub messages: HashMap<String, String>,
}

/// The central registry that holds all translation bundles.
///
/// Supports multiple locales simultaneously. When bundles for the same locale
/// are added, later entries overwrite earlier ones for the same key.
///
/// Lookups are O(1) per locale and O(1) per key.
#[derive(Clone, Debug, Default)]
pub struct I18nRegistry {
    bundles: HashMap<Locale, HashMap<String, String>>,
}

impl I18nRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a translation bundle. Messages are merged into any existing bundle
    /// for the same locale (later values overwrite).
    pub fn add_bundle(&mut self, bundle: TranslationBundle) {
        let entry = self.bundles.entry(bundle.locale).or_default();
        entry.extend(bundle.messages);
    }

    /// Look up a translated string by locale and key.
    ///
    /// Returns `None` if the locale or key is not registered.
    pub fn get(&self, locale: &Locale, key: &str) -> Option<&str> {
        self.bundles
            .get(locale)
            .and_then(|msgs| msgs.get(key).map(|s| s.as_str()))
    }
}
