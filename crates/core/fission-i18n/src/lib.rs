use std::collections::HashMap;
use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct TranslationBundle {
    pub locale: Locale,
    pub messages: HashMap<String, String>,
}

#[derive(Clone, Debug, Default)]
pub struct I18nRegistry {
    // Map Locale -> (Key -> Value)
    bundles: HashMap<Locale, HashMap<String, String>>, 
}

impl I18nRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_bundle(&mut self, bundle: TranslationBundle) {
        let entry = self.bundles.entry(bundle.locale).or_default();
        entry.extend(bundle.messages);
    }

    pub fn get(&self, locale: &Locale, key: &str) -> Option<&str> {
        self.bundles.get(locale).and_then(|msgs| msgs.get(key).map(|s| s.as_str()))
    }
}
