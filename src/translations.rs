use lazy_static::lazy_static;
use log::{debug, warn};
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::io::{self, BufRead, BufReader};
use std::sync::{Arc, Mutex};
use sys_locale::get_locale;

/// Path to the English translations file.
const TRANSLATIONS_EN: &str = include_str!("../locales/en.txt");
/// Path to the Italian translations file.
const TRANSLATIONS_IT: &str = include_str!("../locales/it.txt");

lazy_static! {
    /// Static reference to the translations object, initialized lazily.
    static ref TRANSLATIONS: Arc<Mutex<Translations>> = Arc::new(Mutex::new({
        let mut t = Translations::new();
        t.init().expect("Failed to initialize translations");
        t
    }));
    /// Regular expression to match locale strings.
    static ref LOCALE_REGEX: Regex = Regex::new(r"^([a-z]{2})[-_]?.*$").unwrap();
}

/// Struct representing a set of translations.
#[derive(Debug)]
struct TranslationSet {
    /// Map of translations.
    translations: Arc<HashMap<String, String>>,
    /// Set of missing translation keys.
    missing_keys: HashSet<String>,
}

/// Struct representing the translations with current and fallback sets.
#[derive(Debug)]
pub struct Translations {
    /// Current set of translations.
    current: TranslationSet,
    /// Fallback set of translations.
    fallback: TranslationSet,
}

impl Default for Translations {
    fn default() -> Self {
        Self::new()
    }
}

impl TranslationSet {
    /// Creates a new `TranslationSet`.
    fn new() -> Self {
        TranslationSet {
            translations: Arc::new(HashMap::new()),
            missing_keys: HashSet::new(),
        }
    }

    /// Tracks a missing translation key.
    fn track_missing_key(&mut self, key: &str) {
        self.missing_keys.insert(key.to_string());
    }

    /// Gets the list of missing translation keys.
    fn get_missing_keys(&self) -> Vec<String> {
        self.missing_keys.iter().cloned().collect()
    }
}

impl Translations {
    /// Creates a new `Translations` object.
    pub fn new() -> Self {
        Translations {
            current: TranslationSet::new(),
            fallback: TranslationSet::new(),
        }
    }

    /// Gets the singleton instance of `Translations`.
    pub fn get_instance() -> Arc<Mutex<Translations>> {
        TRANSLATIONS.clone()
    }

    /// Formats a translation string with the given arguments.
    pub fn format(&mut self, key: &str, args: &[&str]) -> String {
        let template = self.get_or_default(key, key);

        let placeholder_count = (0..args.len())
            .map(|i| format!("{{{}}}", i))
            .filter(|p| template.contains(p))
            .count();

        if placeholder_count != args.len() {
            warn!(
                "Mismatch in placeholder count for key '{}'. Expected {}, found {}",
                key,
                args.len(),
                placeholder_count
            );
        }

        args.iter().enumerate().fold(template, |acc, (i, arg)| {
            acc.replace(&format!("{{{}}}", i), arg)
        })
    }

    /// Formats a translation string with the given arguments, converting them to strings.
    pub fn format_display<T: std::fmt::Display>(&mut self, key: &str, args: &[T]) -> String {
        let strings: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
        let str_slices: Vec<&str> = strings.iter().map(|s| s.as_str()).collect();
        self.format(key, &str_slices)
    }

    /// Initializes the translations from the locale.
    pub fn init(&mut self) -> io::Result<()> {
        let mut fallback_map = HashMap::new();
        Self::load_into_map(
            &mut fallback_map,
            BufReader::new(TRANSLATIONS_EN.as_bytes()),
        )?;
        self.fallback = TranslationSet {
            translations: Arc::new(fallback_map),
            missing_keys: HashSet::new(),
        };

        let mut current_map = HashMap::new();
        if let Some(locale) = get_locale() {
            if let Some(captures) = LOCALE_REGEX.captures(&locale.to_lowercase()) {
                if let Some(lang_code) = captures.get(1) {
                    match lang_code.as_str() {
                        "it" => {
                            Self::load_into_map(
                                &mut current_map,
                                BufReader::new(TRANSLATIONS_IT.as_bytes()),
                            )?;
                            self.validate_translations(&current_map);
                        }
                        _ => current_map = (*self.fallback.translations).clone(),
                    }
                }
            }
        } else {
            current_map = (*self.fallback.translations).clone();
        }

        self.current = TranslationSet {
            translations: Arc::new(current_map),
            missing_keys: HashSet::new(),
        };

        Ok(())
    }

    /// Gets the missing translations for both current and fallback sets.
    pub fn get_missing_translations(&self) -> (Vec<String>, Vec<String>) {
        (
            self.current.get_missing_keys(),
            self.fallback.get_missing_keys(),
        )
    }

    /// Validates the current translations against the fallback translations.
    fn validate_translations(&self, current_map: &HashMap<String, String>) {
        for key in self.fallback.translations.keys() {
            if !current_map.contains_key(key) {
                warn!("Missing translation key '{}' in current language", key);
            }
        }

        for key in current_map.keys() {
            if !self.fallback.translations.contains_key(key) {
                warn!(
                    "Extra translation key '{}' in current language not present in fallback",
                    key
                );
            }
        }
    }

    /// Loads translations from a reader into a map.
    fn load_into_map<R: BufRead>(map: &mut HashMap<String, String>, reader: R) -> io::Result<()> {
        for line in reader.lines() {
            let line = line?;
            if line.trim().is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some((key, value)) = line.split_once('=') {
                let key = key.trim().to_string();
                let value = value.trim().trim_matches('"').to_string();
                map.insert(key, unescape(&value));
            }
        }
        Ok(())
    }

    /// Gets a translation for the given key.
    pub fn get(&mut self, key: &str) -> Option<String> {
        match self.current.translations.get(key) {
            Some(value) => Some(value.clone()),
            None => {
                self.current.track_missing_key(key);
                match self.fallback.translations.get(key) {
                    Some(value) => {
                        debug!(
                            "Key '{}' not found in current language, using fallback",
                            key
                        );
                        Some(value.clone())
                    }
                    None => {
                        self.fallback.track_missing_key(key);
                        warn!("Translation key '{}' not found in any language", key);
                        None
                    }
                }
            }
        }
    }

    /// Gets a translation for the given key, returning a default value if not found.
    pub fn get_or_default(&mut self, key: &str, default: &str) -> String {
        self.get(key).unwrap_or_else(|| default.to_string())
    }
}

/// Macro to simplify translation lookups.
#[macro_export]
macro_rules! tr {
    ($translations:expr, $method:ident, $key:expr, $args:expr) => {{
        let mut translations_lock = $translations
            .lock()
            .expect("Failed to acquire translations lock");
        translations_lock.$method($key, $args)
    }};
    ($translations:expr, $method:ident, $key:expr) => {{
        let mut translations_lock = $translations
            .lock()
            .expect("Failed to acquire translations lock");
        translations_lock.$method($key)
    }};
}

/// Unescapes a string by replacing escape sequences with their corresponding characters.
fn unescape(s: &str) -> String {
    let mut result = String::new();
    let mut chars = s.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c == '\\' {
            chars.next();
            if let Some(&next) = chars.peek() {
                match next {
                    'n' => result.push('\n'),
                    't' => result.push('\t'),
                    'r' => result.push('\r'),
                    '\\' => result.push('\\'),
                    '"' => result.push('"'),
                    '\'' => result.push('\''),
                    _ => result.push('\\'),
                }
                chars.next();
            } else {
                result.push('\\');
            }
        } else {
            result.push(c);
            chars.next();
        }
    }
    result
}
