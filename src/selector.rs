use std::ffi::CStr;

use arrow::{
    array::{cast::AsArray, Array, BooleanBuilder, Int64Builder, LargeStringBuilder},
    datatypes::{DataType, Field},
};
use daft_ext::prelude::*;
use scraper::{Html, Selector};

use crate::ffi::{
    apply_string_map, decode_input, encode_output, require_string_arg, scalar_string,
};

// ── html_extract_text ────────────────────────────────────────────────────────

pub(crate) struct ExtractTextFn;

impl DaftScalarFunction for ExtractTextFn {
    fn name(&self) -> &CStr {
        c"html_extract_text"
    }

    fn return_field(&self, args: &[ArrowSchema]) -> DaftResult<ArrowSchema> {
        if args.len() != 2 {
            return Err(DaftError::TypeError(format!(
                "html_extract_text: expected 2 arguments (html_col, selector), got {}",
                args.len()
            )));
        }
        let field = require_string_arg(args, 0, "html_extract_text")?;
        require_string_arg(args, 1, "html_extract_text")?;
        Ok(ArrowSchema::try_from(&Field::new(
            field.name(),
            DataType::LargeUtf8,
            true,
        ))?)
    }

    fn call(&self, args: Vec<ArrowData>) -> DaftResult<ArrowData> {
        let mut iter = args.into_iter();
        let html_data = iter.next().ok_or_else(|| {
            DaftError::RuntimeError("html_extract_text: missing html argument".into())
        })?;
        let sel_data = iter.next().ok_or_else(|| {
            DaftError::RuntimeError("html_extract_text: missing selector argument".into())
        })?;
        let (sel_array, _) = decode_input(sel_data)?;
        let selector_str = scalar_string(&sel_array, "html_extract_text: selector")?;
        let selector = Selector::parse(&selector_str).map_err(|e| {
            DaftError::RuntimeError(format!(
                "html_extract_text: invalid CSS selector {selector_str:?}: {e}"
            ))
        })?;
        let (input, in_field) = decode_input(html_data)?;
        let len = input.len();
        let mut builder = LargeStringBuilder::with_capacity(len, len * 64);
        apply_string_map(
            &input,
            len,
            |s| {
                let doc = Html::parse_document(s);
                doc.select(&selector)
                    .next()
                    .map(|el| el.text().collect::<String>().trim().to_string())
                    .filter(|t| !t.is_empty())
            },
            &mut builder,
        )?;
        let output = builder.finish();
        let out_field = Field::new(in_field.name(), DataType::LargeUtf8, true);
        encode_output(&output.to_data(), &out_field)
    }
}

// ── html_get_attribute ───────────────────────────────────────────────────────

pub(crate) struct GetAttributeFn;

impl DaftScalarFunction for GetAttributeFn {
    fn name(&self) -> &CStr {
        c"html_get_attribute"
    }

    fn return_field(&self, args: &[ArrowSchema]) -> DaftResult<ArrowSchema> {
        if args.len() != 3 {
            return Err(DaftError::TypeError(format!(
                "html_get_attribute: expected 3 arguments (html_col, selector, attr), got {}",
                args.len()
            )));
        }
        let field = require_string_arg(args, 0, "html_get_attribute")?;
        require_string_arg(args, 1, "html_get_attribute")?;
        require_string_arg(args, 2, "html_get_attribute")?;
        Ok(ArrowSchema::try_from(&Field::new(
            field.name(),
            DataType::LargeUtf8,
            true,
        ))?)
    }

    fn call(&self, args: Vec<ArrowData>) -> DaftResult<ArrowData> {
        let mut iter = args.into_iter();
        let html_data = iter.next().ok_or_else(|| {
            DaftError::RuntimeError("html_get_attribute: missing html argument".into())
        })?;
        let sel_data = iter.next().ok_or_else(|| {
            DaftError::RuntimeError("html_get_attribute: missing selector argument".into())
        })?;
        let attr_data = iter.next().ok_or_else(|| {
            DaftError::RuntimeError("html_get_attribute: missing attr argument".into())
        })?;
        let (sel_array, _) = decode_input(sel_data)?;
        let (attr_array, _) = decode_input(attr_data)?;
        let selector_str = scalar_string(&sel_array, "html_get_attribute: selector")?;
        let attr_name = scalar_string(&attr_array, "html_get_attribute: attr")?;
        let selector = Selector::parse(&selector_str).map_err(|e| {
            DaftError::RuntimeError(format!(
                "html_get_attribute: invalid CSS selector {selector_str:?}: {e}"
            ))
        })?;
        let (input, in_field) = decode_input(html_data)?;
        let len = input.len();
        let mut builder = LargeStringBuilder::with_capacity(len, len * 64);
        apply_string_map(
            &input,
            len,
            |s| {
                let doc = Html::parse_document(s);
                doc.select(&selector)
                    .next()
                    .and_then(|el| el.value().attr(&attr_name).map(str::to_string))
            },
            &mut builder,
        )?;
        let output = builder.finish();
        let out_field = Field::new(in_field.name(), DataType::LargeUtf8, true);
        encode_output(&output.to_data(), &out_field)
    }
}

// ── html_has_element ─────────────────────────────────────────────────────────

pub(crate) struct HasElementFn;

impl DaftScalarFunction for HasElementFn {
    fn name(&self) -> &CStr {
        c"html_has_element"
    }

    fn return_field(&self, args: &[ArrowSchema]) -> DaftResult<ArrowSchema> {
        if args.len() != 2 {
            return Err(DaftError::TypeError(format!(
                "html_has_element: expected 2 arguments (html_col, selector), got {}",
                args.len()
            )));
        }
        let field = require_string_arg(args, 0, "html_has_element")?;
        require_string_arg(args, 1, "html_has_element")?;
        Ok(ArrowSchema::try_from(&Field::new(
            field.name(),
            DataType::Boolean,
            field.is_nullable(),
        ))?)
    }

    fn call(&self, args: Vec<ArrowData>) -> DaftResult<ArrowData> {
        let mut iter = args.into_iter();
        let html_data = iter.next().ok_or_else(|| {
            DaftError::RuntimeError("html_has_element: missing html argument".into())
        })?;
        let sel_data = iter.next().ok_or_else(|| {
            DaftError::RuntimeError("html_has_element: missing selector argument".into())
        })?;
        let (sel_array, _) = decode_input(sel_data)?;
        let selector_str = scalar_string(&sel_array, "html_has_element: selector")?;
        let selector = Selector::parse(&selector_str).map_err(|e| {
            DaftError::RuntimeError(format!(
                "html_has_element: invalid CSS selector {selector_str:?}: {e}"
            ))
        })?;
        let (input, in_field) = decode_input(html_data)?;
        let len = input.len();
        let mut builder = BooleanBuilder::with_capacity(len);
        match input.data_type() {
            DataType::Utf8 => {
                let arr = input.as_string::<i32>();
                for i in 0..len {
                    if arr.is_null(i) {
                        builder.append_null();
                    } else {
                        let doc = Html::parse_document(arr.value(i));
                        builder.append_value(doc.select(&selector).next().is_some());
                    }
                }
            }
            DataType::LargeUtf8 => {
                let arr = input.as_string::<i64>();
                for i in 0..len {
                    if arr.is_null(i) {
                        builder.append_null();
                    } else {
                        let doc = Html::parse_document(arr.value(i));
                        builder.append_value(doc.select(&selector).next().is_some());
                    }
                }
            }
            dt => {
                return Err(DaftError::RuntimeError(format!(
                    "expected String/LargeUtf8, got {dt:?}"
                )))
            }
        }
        let output = builder.finish();
        let out_field = Field::new(in_field.name(), DataType::Boolean, in_field.is_nullable());
        encode_output(&output.to_data(), &out_field)
    }
}

// ── html_count_elements ──────────────────────────────────────────────────────

pub(crate) struct CountElementsFn;

impl DaftScalarFunction for CountElementsFn {
    fn name(&self) -> &CStr {
        c"html_count_elements"
    }

    fn return_field(&self, args: &[ArrowSchema]) -> DaftResult<ArrowSchema> {
        if args.len() != 2 {
            return Err(DaftError::TypeError(format!(
                "html_count_elements: expected 2 arguments (html_col, selector), got {}",
                args.len()
            )));
        }
        let field = require_string_arg(args, 0, "html_count_elements")?;
        require_string_arg(args, 1, "html_count_elements")?;
        Ok(ArrowSchema::try_from(&Field::new(
            field.name(),
            DataType::Int64,
            field.is_nullable(),
        ))?)
    }

    fn call(&self, args: Vec<ArrowData>) -> DaftResult<ArrowData> {
        let mut iter = args.into_iter();
        let html_data = iter.next().ok_or_else(|| {
            DaftError::RuntimeError("html_count_elements: missing html argument".into())
        })?;
        let sel_data = iter.next().ok_or_else(|| {
            DaftError::RuntimeError("html_count_elements: missing selector argument".into())
        })?;
        let (sel_array, _) = decode_input(sel_data)?;
        let selector_str = scalar_string(&sel_array, "html_count_elements: selector")?;
        let selector = Selector::parse(&selector_str).map_err(|e| {
            DaftError::RuntimeError(format!(
                "html_count_elements: invalid CSS selector {selector_str:?}: {e}"
            ))
        })?;
        let (input, in_field) = decode_input(html_data)?;
        let len = input.len();
        let mut builder = Int64Builder::with_capacity(len);
        match input.data_type() {
            DataType::Utf8 => {
                let arr = input.as_string::<i32>();
                for i in 0..len {
                    if arr.is_null(i) {
                        builder.append_null();
                    } else {
                        let doc = Html::parse_document(arr.value(i));
                        builder.append_value(doc.select(&selector).count() as i64);
                    }
                }
            }
            DataType::LargeUtf8 => {
                let arr = input.as_string::<i64>();
                for i in 0..len {
                    if arr.is_null(i) {
                        builder.append_null();
                    } else {
                        let doc = Html::parse_document(arr.value(i));
                        builder.append_value(doc.select(&selector).count() as i64);
                    }
                }
            }
            dt => {
                return Err(DaftError::RuntimeError(format!(
                    "expected String/LargeUtf8, got {dt:?}"
                )))
            }
        }
        let output = builder.finish();
        let out_field = Field::new(in_field.name(), DataType::Int64, in_field.is_nullable());
        encode_output(&output.to_data(), &out_field)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_text_basic() {
        let doc = Html::parse_document(r#"<div id="main"><p>Hello CSS</p></div>"#);
        let sel = Selector::parse("#main p").unwrap();
        let text: String = doc.select(&sel).next().unwrap().text().collect();
        assert_eq!(text.trim(), "Hello CSS");
    }

    #[test]
    fn test_extract_text_missing() {
        let doc = Html::parse_document("<p>no match</p>");
        let sel = Selector::parse(".nonexistent").unwrap();
        assert!(doc.select(&sel).next().is_none());
    }

    #[test]
    fn test_get_attribute_basic() {
        let doc = Html::parse_document(r#"<img src="photo.jpg" alt="a photo">"#);
        let sel = Selector::parse("img").unwrap();
        let src = doc.select(&sel).next().unwrap().value().attr("src");
        assert_eq!(src, Some("photo.jpg"));
    }

    #[test]
    fn test_has_element_true() {
        let doc = Html::parse_document(r#"<div class="hero"><h1>Title</h1></div>"#);
        let sel = Selector::parse(".hero").unwrap();
        assert!(doc.select(&sel).next().is_some());
    }

    #[test]
    fn test_has_element_false() {
        let doc = Html::parse_document("<p>no hero here</p>");
        let sel = Selector::parse(".hero").unwrap();
        assert!(doc.select(&sel).next().is_none());
    }

    #[test]
    fn test_count_elements() {
        let doc = Html::parse_document("<ul><li>a</li><li>b</li><li>c</li></ul>");
        let sel = Selector::parse("li").unwrap();
        assert_eq!(doc.select(&sel).count(), 3);
    }

    #[test]
    fn test_count_elements_zero() {
        let doc = Html::parse_document("<p>no list</p>");
        let sel = Selector::parse("li").unwrap();
        assert_eq!(doc.select(&sel).count(), 0);
    }
}
