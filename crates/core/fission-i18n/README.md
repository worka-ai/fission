# fission-i18n

Internationalization (i18n) support for Fission applications.

This crate provides a simple, registry-based translation system. It stores locale-specific string bundles and retrieves messages by key and locale at runtime.

## Core types

### `Locale`

A newtype wrapper around `String` representing a BCP 47 locale identifier (e.g., `"en-US"`, `"ja-JP"`). Defaults to `"en-US"`.

```rust
use fission_i18n::Locale;

let locale = Locale::default();          // "en-US"
let locale = Locale::from("fr-FR");      // French
let locale = Locale("de-DE".to_string()); // German
```

### `TranslationBundle`

A collection of key-value message pairs for a single locale. Load these from JSON, TOML, or construct them programmatically.

```rust
use fission_i18n::{TranslationBundle, Locale};
use std::collections::HashMap;

let bundle = TranslationBundle {
    locale: Locale::from("es-ES"),
    messages: HashMap::from([
        ("greeting".to_string(), "Hola".to_string()),
        ("farewell".to_string(), "Adios".to_string()),
    ]),
};
```

### `I18nRegistry`

The central registry that holds all translation bundles. Supports multiple locales simultaneously and merges bundles for the same locale (later entries overwrite earlier ones for the same key).

```rust
use fission_i18n::{I18nRegistry, TranslationBundle, Locale};

let mut registry = I18nRegistry::new();
registry.add_bundle(english_bundle);
registry.add_bundle(spanish_bundle);

let locale = Locale::from("es-ES");
let greeting = registry.get(&locale, "greeting"); // Some("Hola")
let missing = registry.get(&locale, "unknown");   // None
```

## Serialization

All types implement `Serialize` and `Deserialize`, so bundles can be loaded from JSON files:

```json
{
  "locale": "ja-JP",
  "messages": {
    "save": "保存",
    "cancel": "キャンセル",
    "delete": "削除"
  }
}
```

## Design notes

- The registry uses `HashMap<Locale, HashMap<String, String>>` internally.
- Lookups are O(1) per locale and O(1) per key.
- There is no fallback chain (e.g., `en-US` does not fall back to `en`). The caller is responsible for implementing fallback logic if needed.
- The registry is not global -- it is owned by the application and passed through the `Env` or state.
