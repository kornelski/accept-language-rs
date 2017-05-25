//! `accept-language` is a tiny library for parsing the Accept-Language header from browsers (as defined [here](https://www.w3.org/Protocols/rfc2616/rfc2616-sec14.html)).
//!
//! It's intended to be used in a webserver that supports some level of internationalization (i18n),
//! but can be used anytime an Accept-Language header string is available.
//!
//! In order to help facilitate better i18n, a function is provided to return the intersection of
//! the languages the user prefers and the languages your application supports.
//!
//! # Example
//!
//! ```
//! use accept_language::{intersection, parse};
//!
//! let user_languages = parse("en-US, en-GB;q=0.5");
//! let common_languages = intersection("en-US, en-GB;q=0.5", vec!["en-US", "de", "en-GB"]);
//! ```
use std::str;
use std::str::FromStr;
use std::cmp::Ordering;

#[derive(Debug)]
struct Language {
    name: String,
    quality: f64
}

impl Eq for Language {}

impl Ord for Language {
    fn cmp(&self, other: &Language) -> Ordering {
        if self.quality > other.quality {
            Ordering::Less
        } else if self.quality < other.quality {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    }
}

impl PartialOrd for Language {
    fn partial_cmp(&self, other: &Language) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Language {
    fn eq(&self, other: &Language) -> bool {
        self.quality == other.quality
    }
}

impl Language {
    fn new(tag: &str) -> Language {
        let mut tag_parts = tag.split(";");
        let name = match tag_parts.nth(0) {
            Some(name_str) => name_str.to_string(),
            None => String::from("")
        };
        let quality = match tag_parts.nth(0) {
            Some(quality_str) => Language::quality_with_default(quality_str),
            None => 1.0
        };

        Language {
            name: name,
            quality: quality
        }
    }

    fn quality_with_default(raw_quality: &str) -> f64 {
        let quality_str = &raw_quality[2..];

        match f64::from_str(&quality_str) {
            Ok(q) => q,
            Err(_) => 0.0
        }
    }
}

/// Parse a raw Accept-Language header value into an ordered list of language tags.
/// This should return the exact same list as `window.navigator.languages` in supported browsers.
///
/// # Example
///
/// ```
/// use accept_language::parse;
///
/// let user_languages = parse("en-US, en-GB;q=0.5");
/// ```
pub fn parse(raw_languages: &str) -> Vec<String> {
    let languages_string = raw_languages.clone().replace(" ", "");
    let languages_str_parts: Vec<&str> = languages_string.split(",").collect();
    let mut languages_string_parts: Vec<Language> = languages_str_parts
        .iter()
        .map(|l| Language::new(l))
        .collect();

    languages_string_parts.sort();

    languages_string_parts
        .iter()
        .map(|ref l| l.name.to_owned())
        .filter(|l| !l.is_empty())
        .collect()
}

/// Compare an Accept-Language header value with your application's supported languages to find
/// the common languages that could be presented to a user.
///
/// # Example
///
/// ```
/// use accept_language::intersection;
///
/// let common_languages = intersection("en-US, en-GB;q=0.5", vec!["en-US", "de", "en-GB"]);
/// ```
pub fn intersection(raw_languages: &str, supported_languages: Vec<&str>) -> Vec<String> {
    let user_languages = parse(raw_languages);
    let intersection = user_languages
        .into_iter()
        .filter(|l| supported_languages.contains(&l.as_str()))
        .map(|l| l.to_string())
        .collect();

    intersection
}

#[cfg(test)]
mod tests {
    use super::{intersection, Language, parse};

    static MOCK_ACCEPT_LANGUAGE: &str = "en-US, de;q=0.7, jp;q=0.1";

    #[test]
    fn it_creates_a_new_language_from_a_string() {
        let language = Language::new("en-US;q=0.7");

        assert_eq!(language, Language { name: String::from("en-US"), quality: 0.7 })
    }

    #[test]
    fn it_creates_a_new_language_from_a_string_with_a_default_quality() {
        let language = Language::new("en-US");

        assert_eq!(language, Language { name: String::from("en-US"), quality: 1.0 })
    }

    #[test]
    fn it_parses_quality() {
        let quality = Language::quality_with_default("q=0.5");

        assert_eq!(quality, 0.5)
    }

    #[test]
    fn it_parses_an_invalid_quality() {
        let quality = Language::quality_with_default("q=yolo");

        assert_eq!(quality, 0.0)
    }

    #[test]
    fn it_parses_a_valid_accept_language_header() {
        let user_languages = parse(MOCK_ACCEPT_LANGUAGE);

        assert_eq!(user_languages, vec![String::from("en-US"), String::from("de"), String::from("jp")])
    }

    #[test]
    fn it_parses_an_empty_accept_language_header() {
        let user_languages = parse("");

        assert_eq!(user_languages.len(), 0)
    }

    #[test]
    fn it_sorts_languages_by_quality() {
        let user_languages = parse("en-US, de;q=0.1, jp;q=0.7");

        assert_eq!(user_languages, vec![String::from("en-US"), String::from("jp"), String::from("de")])
    }

    #[test]
    fn it_returns_language_intersections() {
        let common_languages = intersection(MOCK_ACCEPT_LANGUAGE, vec!["en-US", "jp"]);

        assert_eq!(common_languages, vec![String::from("en-US"), String::from("jp")])
    }

    #[test]
    fn it_returns_an_empty_array_when_no_intersections() {
        let common_languages = intersection(MOCK_ACCEPT_LANGUAGE, vec!["fr", "en-GB"]);

        assert_eq!(common_languages.len(), 0)
    }
}
