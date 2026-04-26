//! daft-html — Daft native extension for HTML processing.
//!
//! Implements HTML operators (parsing, extraction, transformation) in Rust via
//! the `daft-ext` ABI and exposes them as Daft scalar functions.
//!
//! All operators use [`scraper`] for DOM parsing, which correctly handles HTML5
//! error recovery (foster parenting, implicit element insertion, etc.).
//!
//! # Registered operators
//!
//! ## Document-level extraction
//!
//! | Function name         | Signature                          | Description                                   |
//! |-----------------------|------------------------------------|-----------------------------------------------|
//! | `html_to_text`        | String → String                    | Extract plain text, discard tags              |
//! | `html_get_title`      | String → String                    | Extract `<title>` text                        |
//! | `html_text_ratio`     | String → Float64                   | Ratio of visible text chars to raw HTML bytes |
//! | `html_extract_meta`   | (String, String) → String          | `<meta name\|property="…" content>` value     |
//! | `html_extract_links`  | String → List[String]              | All `<a href="…">` URLs                       |
//! | `html_extract_tables` | String → List[String]              | `<table>` elements rendered as Markdown       |
//!
//! ## CSS-selector operators
//!
//! | Function name          | Signature                          | Description                                   |
//! |------------------------|------------------------------------|-----------------------------------------------|
//! | `html_extract_text`    | (String, String) → String          | First match for selector → inner text         |
//! | `html_get_attribute`   | (String, String, String) → String  | First match for selector → attribute value    |
//! | `html_has_element`     | (String, String) → Bool            | True if selector matches at least one element |
//! | `html_count_elements`  | (String, String) → Int64           | Number of elements matching selector          |

mod document;
mod ffi;
mod selector;

use std::sync::Arc;

use daft_ext::{daft_extension, prelude::*};

use document::{
    ExtractLinksFn, ExtractMetaFn, ExtractTablesFn, GetTitleFn, HtmlToTextFn, TextRatioFn,
};
use selector::{CountElementsFn, ExtractTextFn, GetAttributeFn, HasElementFn};

#[daft_extension]
struct DaftHtmlExtension;

impl DaftExtension for DaftHtmlExtension {
    fn install(session: &mut dyn DaftSession) {
        session.define_function(Arc::new(HtmlToTextFn));
        session.define_function(Arc::new(GetTitleFn));
        session.define_function(Arc::new(TextRatioFn));
        session.define_function(Arc::new(ExtractMetaFn));
        session.define_function(Arc::new(ExtractLinksFn));
        session.define_function(Arc::new(ExtractTablesFn));
        session.define_function(Arc::new(ExtractTextFn));
        session.define_function(Arc::new(GetAttributeFn));
        session.define_function(Arc::new(HasElementFn));
        session.define_function(Arc::new(CountElementsFn));
    }
}
