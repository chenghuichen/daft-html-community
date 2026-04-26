use std::{ffi::CStr, sync::Arc};

use arrow::{
    array::{cast::AsArray, Array, Float64Builder, LargeStringBuilder, ListBuilder},
    datatypes::{DataType, Field},
};
use daft_ext::prelude::*;
use scraper::{node::Node, ElementRef, Html, Selector};

use crate::ffi::{
    apply_list_map, apply_string_map, decode_input, encode_output, require_string_arg,
    scalar_string,
};

// ── DOM text extraction helpers ──────────────────────────────────────────────

const SKIP_TAGS: &[&str] = &["script", "style", "head", "noscript", "template"];

const BLOCK_TAGS: &[&str] = &[
    "p",
    "div",
    "br",
    "h1",
    "h2",
    "h3",
    "h4",
    "h5",
    "h6",
    "li",
    "tr",
    "blockquote",
    "pre",
    "article",
    "section",
    "header",
    "footer",
    "main",
    "nav",
    "aside",
    "dl",
    "dt",
    "dd",
];

fn collect_text(el: ElementRef<'_>, out: &mut String, sep: &str) {
    let tag = el.value().name();
    if SKIP_TAGS.contains(&tag) {
        return;
    }
    let is_block = BLOCK_TAGS.contains(&tag);
    if is_block && !out.is_empty() && !out.ends_with(sep) {
        out.push_str(sep);
    }
    for child in el.children() {
        match child.value() {
            Node::Text(t) => out.push_str(t),
            Node::Element(_) => {
                if let Some(child_el) = ElementRef::wrap(child) {
                    collect_text(child_el, out, sep);
                }
            }
            _ => {}
        }
    }
}

// ── html_to_text ─────────────────────────────────────────────────────────────

pub(crate) struct HtmlToTextFn;

impl DaftScalarFunction for HtmlToTextFn {
    fn name(&self) -> &CStr {
        c"html_to_text"
    }

    fn return_field(&self, args: &[ArrowSchema]) -> DaftResult<ArrowSchema> {
        if args.len() != 1 {
            return Err(DaftError::TypeError(format!(
                "html_to_text: expected 1 argument, got {}",
                args.len()
            )));
        }
        let field = require_string_arg(args, 0, "html_to_text")?;
        Ok(ArrowSchema::try_from(&Field::new(
            field.name(),
            DataType::LargeUtf8,
            field.is_nullable(),
        ))?)
    }

    fn call(&self, args: Vec<ArrowData>) -> DaftResult<ArrowData> {
        let data = args
            .into_iter()
            .next()
            .ok_or_else(|| DaftError::RuntimeError("html_to_text: no arguments provided".into()))?;
        let (input, in_field) = decode_input(data)?;
        let len = input.len();
        let mut builder = LargeStringBuilder::with_capacity(len, len * 64);
        apply_string_map(&input, len, |s| Some(strip_html_str(s)), &mut builder)?;
        let output = builder.finish();
        let out_field = Field::new(in_field.name(), DataType::LargeUtf8, in_field.is_nullable());
        encode_output(&output.to_data(), &out_field)
    }
}

pub(crate) fn strip_html_str(html: &str) -> String {
    let doc = Html::parse_document(html);
    let mut out = String::with_capacity(html.len() / 2);
    collect_text(doc.root_element(), &mut out, "\n");
    out.trim().to_string()
}

// ── html_get_title ───────────────────────────────────────────────────────────

pub(crate) struct GetTitleFn;

impl DaftScalarFunction for GetTitleFn {
    fn name(&self) -> &CStr {
        c"html_get_title"
    }

    fn return_field(&self, args: &[ArrowSchema]) -> DaftResult<ArrowSchema> {
        if args.len() != 1 {
            return Err(DaftError::TypeError(format!(
                "html_get_title: expected 1 argument, got {}",
                args.len()
            )));
        }
        let field = require_string_arg(args, 0, "html_get_title")?;
        Ok(ArrowSchema::try_from(&Field::new(
            field.name(),
            DataType::LargeUtf8,
            true,
        ))?)
    }

    fn call(&self, args: Vec<ArrowData>) -> DaftResult<ArrowData> {
        let data = args.into_iter().next().ok_or_else(|| {
            DaftError::RuntimeError("html_get_title: no arguments provided".into())
        })?;
        let (input, in_field) = decode_input(data)?;
        let len = input.len();
        let mut builder = LargeStringBuilder::with_capacity(len, len * 32);
        apply_string_map(&input, len, get_title_str, &mut builder)?;
        let output = builder.finish();
        let out_field = Field::new(in_field.name(), DataType::LargeUtf8, true);
        encode_output(&output.to_data(), &out_field)
    }
}

fn get_title_str(html: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let sel = Selector::parse("title").unwrap();
    doc.select(&sel)
        .next()
        .map(|el| {
            let text: String = el.text().collect();
            text.trim().to_string()
        })
        .filter(|s| !s.is_empty())
}

// ── html_text_ratio ──────────────────────────────────────────────────────────

pub(crate) struct TextRatioFn;

impl DaftScalarFunction for TextRatioFn {
    fn name(&self) -> &CStr {
        c"html_text_ratio"
    }

    fn return_field(&self, args: &[ArrowSchema]) -> DaftResult<ArrowSchema> {
        if args.len() != 1 {
            return Err(DaftError::TypeError(format!(
                "html_text_ratio: expected 1 argument, got {}",
                args.len()
            )));
        }
        let field = require_string_arg(args, 0, "html_text_ratio")?;
        Ok(ArrowSchema::try_from(&Field::new(
            field.name(),
            DataType::Float64,
            field.is_nullable(),
        ))?)
    }

    fn call(&self, args: Vec<ArrowData>) -> DaftResult<ArrowData> {
        let data = args.into_iter().next().ok_or_else(|| {
            DaftError::RuntimeError("html_text_ratio: no arguments provided".into())
        })?;
        let (input, in_field) = decode_input(data)?;
        let len = input.len();
        let mut builder = Float64Builder::with_capacity(len);
        match input.data_type() {
            DataType::Utf8 => {
                let arr = input.as_string::<i32>();
                for i in 0..len {
                    if arr.is_null(i) {
                        builder.append_null();
                    } else {
                        builder.append_value(text_ratio_str(arr.value(i)));
                    }
                }
            }
            DataType::LargeUtf8 => {
                let arr = input.as_string::<i64>();
                for i in 0..len {
                    if arr.is_null(i) {
                        builder.append_null();
                    } else {
                        builder.append_value(text_ratio_str(arr.value(i)));
                    }
                }
            }
            dt => {
                return Err(DaftError::RuntimeError(format!(
                    "html_text_ratio: expected string array, got {dt:?}"
                )))
            }
        }
        let output = builder.finish();
        let out_field = Field::new(in_field.name(), DataType::Float64, in_field.is_nullable());
        encode_output(&output.to_data(), &out_field)
    }
}

fn text_ratio_str(html: &str) -> f64 {
    let total = html.len();
    if total == 0 {
        return 0.0;
    }
    let text = strip_html_str(html);
    let visible_chars = text.chars().filter(|c| !c.is_whitespace()).count();
    visible_chars as f64 / total as f64
}

// ── html_extract_meta ────────────────────────────────────────────────────────

pub(crate) struct ExtractMetaFn;

impl DaftScalarFunction for ExtractMetaFn {
    fn name(&self) -> &CStr {
        c"html_extract_meta"
    }

    fn return_field(&self, args: &[ArrowSchema]) -> DaftResult<ArrowSchema> {
        if args.len() != 2 {
            return Err(DaftError::TypeError(format!(
                "html_extract_meta: expected 2 arguments (html_col, meta_name), got {}",
                args.len()
            )));
        }
        let field = require_string_arg(args, 0, "html_extract_meta")?;
        require_string_arg(args, 1, "html_extract_meta")?;
        Ok(ArrowSchema::try_from(&Field::new(
            field.name(),
            DataType::LargeUtf8,
            true,
        ))?)
    }

    fn call(&self, args: Vec<ArrowData>) -> DaftResult<ArrowData> {
        let mut iter = args.into_iter();
        let html_data = iter.next().ok_or_else(|| {
            DaftError::RuntimeError("html_extract_meta: missing html argument".into())
        })?;
        let name_data = iter.next().ok_or_else(|| {
            DaftError::RuntimeError("html_extract_meta: missing meta_name argument".into())
        })?;
        let (name_array, _) = decode_input(name_data)?;
        let meta_name = scalar_string(&name_array, "html_extract_meta: meta_name")?;
        let (input, in_field) = decode_input(html_data)?;
        let len = input.len();
        let mut builder = LargeStringBuilder::with_capacity(len, len * 64);
        apply_string_map(
            &input,
            len,
            |s| extract_meta_str(s, &meta_name),
            &mut builder,
        )?;
        let output = builder.finish();
        let out_field = Field::new(in_field.name(), DataType::LargeUtf8, true);
        encode_output(&output.to_data(), &out_field)
    }
}

fn extract_meta_str(html: &str, name: &str) -> Option<String> {
    let doc = Html::parse_document(html);
    let sel = Selector::parse("meta").unwrap();
    for el in doc.select(&sel) {
        let val = el.value();
        let matches_name = val.attr("name").is_some_and(|v| v == name)
            || val.attr("property").is_some_and(|v| v == name);
        if matches_name {
            if let Some(content) = val.attr("content") {
                return Some(content.to_string());
            }
        }
    }
    None
}

// ── html_extract_links ───────────────────────────────────────────────────────

pub(crate) struct ExtractLinksFn;

impl DaftScalarFunction for ExtractLinksFn {
    fn name(&self) -> &CStr {
        c"html_extract_links"
    }

    fn return_field(&self, args: &[ArrowSchema]) -> DaftResult<ArrowSchema> {
        if args.len() != 1 {
            return Err(DaftError::TypeError(format!(
                "html_extract_links: expected 1 argument, got {}",
                args.len()
            )));
        }
        let field = require_string_arg(args, 0, "html_extract_links")?;
        let item_field = Arc::new(Field::new("item", DataType::LargeUtf8, true));
        Ok(ArrowSchema::try_from(&Field::new(
            field.name(),
            DataType::List(item_field),
            true,
        ))?)
    }

    fn call(&self, args: Vec<ArrowData>) -> DaftResult<ArrowData> {
        let data = args.into_iter().next().ok_or_else(|| {
            DaftError::RuntimeError("html_extract_links: no arguments provided".into())
        })?;
        let (input, in_field) = decode_input(data)?;
        let len = input.len();
        let mut builder = ListBuilder::new(LargeStringBuilder::new());
        apply_list_map(&input, len, extract_links_str, &mut builder)?;
        let output = builder.finish();
        let item_field = Arc::new(Field::new("item", DataType::LargeUtf8, true));
        let out_field = Field::new(in_field.name(), DataType::List(item_field), true);
        encode_output(&output.to_data(), &out_field)
    }
}

fn extract_links_str(html: &str) -> Vec<String> {
    let doc = Html::parse_document(html);
    let sel = Selector::parse("a[href]").unwrap();
    doc.select(&sel)
        .filter_map(|el| el.value().attr("href").map(str::to_string))
        .collect()
}

// ── html_extract_tables ──────────────────────────────────────────────────────

pub(crate) struct ExtractTablesFn;

impl DaftScalarFunction for ExtractTablesFn {
    fn name(&self) -> &CStr {
        c"html_extract_tables"
    }

    fn return_field(&self, args: &[ArrowSchema]) -> DaftResult<ArrowSchema> {
        if args.len() != 1 {
            return Err(DaftError::TypeError(format!(
                "html_extract_tables: expected 1 argument, got {}",
                args.len()
            )));
        }
        let field = require_string_arg(args, 0, "html_extract_tables")?;
        let item_field = Arc::new(Field::new("item", DataType::LargeUtf8, true));
        Ok(ArrowSchema::try_from(&Field::new(
            field.name(),
            DataType::List(item_field),
            true,
        ))?)
    }

    fn call(&self, args: Vec<ArrowData>) -> DaftResult<ArrowData> {
        let data = args.into_iter().next().ok_or_else(|| {
            DaftError::RuntimeError("html_extract_tables: no arguments provided".into())
        })?;
        let (input, in_field) = decode_input(data)?;
        let len = input.len();
        let mut builder = ListBuilder::new(LargeStringBuilder::new());
        apply_list_map(&input, len, extract_tables_str, &mut builder)?;
        let output = builder.finish();
        let item_field = Arc::new(Field::new("item", DataType::LargeUtf8, true));
        let out_field = Field::new(in_field.name(), DataType::List(item_field), true);
        encode_output(&output.to_data(), &out_field)
    }
}

type Row = Vec<(String, bool)>;

fn table_to_rows(table: ElementRef<'_>) -> Vec<Row> {
    let mut rows: Vec<Row> = Vec::new();
    for child in table.children().filter_map(ElementRef::wrap) {
        match child.value().name() {
            "tr" => rows.push(parse_tr(child)),
            "thead" | "tbody" | "tfoot" => {
                for sub in child.children().filter_map(ElementRef::wrap) {
                    if sub.value().name() == "tr" {
                        rows.push(parse_tr(sub));
                    }
                }
            }
            _ => {}
        }
    }
    rows.retain(|r| !r.is_empty());
    rows
}

fn parse_tr(tr: ElementRef<'_>) -> Row {
    tr.children()
        .filter_map(ElementRef::wrap)
        .filter(|el| matches!(el.value().name(), "td" | "th"))
        .map(|el| {
            let is_th = el.value().name() == "th";
            let text = el.text().collect::<String>();
            (text.trim().to_string(), is_th)
        })
        .collect()
}

fn rows_to_markdown(rows: Vec<Row>) -> String {
    if rows.is_empty() {
        return String::new();
    }
    let col_count = rows.iter().map(|r| r.len()).max().unwrap_or(0);
    if col_count == 0 {
        return String::new();
    }
    let mut md = String::new();
    let mut sep_done = false;
    for row in &rows {
        let is_header = row.iter().any(|(_, h)| *h);
        let cells: Vec<String> = (0..col_count)
            .map(|i| {
                row.get(i)
                    .map(|(s, _)| s.replace('|', r"\|"))
                    .unwrap_or_default()
            })
            .collect();
        md.push_str("| ");
        md.push_str(&cells.join(" | "));
        md.push_str(" |\n");
        if is_header && !sep_done {
            md.push('|');
            for _ in 0..col_count {
                md.push_str("---|");
            }
            md.push('\n');
            sep_done = true;
        }
    }
    md
}

fn extract_tables_str(html: &str) -> Vec<String> {
    let doc = Html::parse_document(html);
    let table_sel = Selector::parse("table").unwrap();
    doc.select(&table_sel)
        .filter(|t| {
            t.ancestors()
                .filter_map(ElementRef::wrap)
                .all(|a| a.value().name() != "table")
        })
        .map(|t| rows_to_markdown(table_to_rows(t)))
        .filter(|s| !s.is_empty())
        .collect()
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_html_basic() {
        assert_eq!(strip_html_str("<p>Hello world</p>"), "Hello world");
    }

    #[test]
    fn test_strip_html_script_removed() {
        let result = strip_html_str("<p>text</p><script>alert(1)</script>");
        assert_eq!(result, "text");
    }

    #[test]
    fn test_strip_html_nested() {
        let result = strip_html_str("<div><p>Hello</p><p>World</p></div>");
        assert!(result.contains("Hello"), "got: {result:?}");
        assert!(result.contains("World"), "got: {result:?}");
    }

    #[test]
    fn test_strip_html_empty() {
        assert_eq!(strip_html_str(""), "");
    }

    #[test]
    fn test_get_title_found() {
        assert_eq!(
            get_title_str("<html><head><title>My Page</title></head></html>"),
            Some("My Page".to_string())
        );
    }

    #[test]
    fn test_get_title_missing() {
        assert_eq!(
            get_title_str("<html><body><p>no title</p></body></html>"),
            None
        );
    }

    #[test]
    fn test_get_title_whitespace_trimmed() {
        assert_eq!(
            get_title_str("<title>  spaces  </title>"),
            Some("spaces".to_string())
        );
    }

    #[test]
    fn test_text_ratio_high() {
        let ratio = text_ratio_str("<p>hello world</p>");
        assert!(ratio > 0.3, "got: {ratio}");
    }

    #[test]
    fn test_text_ratio_low() {
        let ratio = text_ratio_str("<div><span><a href='x'></a></span></div>");
        assert!(ratio < 0.1, "got: {ratio}");
    }

    #[test]
    fn test_text_ratio_empty() {
        assert_eq!(text_ratio_str(""), 0.0);
    }

    #[test]
    fn test_extract_meta_description() {
        let html = r#"<meta name="description" content="A great page">"#;
        assert_eq!(
            extract_meta_str(html, "description"),
            Some("A great page".to_string())
        );
    }

    #[test]
    fn test_extract_meta_og_property() {
        let html = r#"<meta property="og:title" content="OG Title">"#;
        assert_eq!(
            extract_meta_str(html, "og:title"),
            Some("OG Title".to_string())
        );
    }

    #[test]
    fn test_extract_meta_missing() {
        assert_eq!(extract_meta_str("<p>no meta</p>", "description"), None);
    }

    #[test]
    fn test_extract_links_basic() {
        let html = r#"<a href="https://example.com">link</a><a href="/path">local</a>"#;
        let links = extract_links_str(html);
        assert_eq!(links, vec!["https://example.com", "/path"]);
    }

    #[test]
    fn test_extract_links_no_href_skipped() {
        let html = r#"<a name="anchor">no href</a><a href="https://x.com">yes</a>"#;
        let links = extract_links_str(html);
        assert_eq!(links, vec!["https://x.com"]);
    }

    #[test]
    fn test_extract_links_empty() {
        assert!(extract_links_str("<p>no links here</p>").is_empty());
    }

    #[test]
    fn test_extract_tables_basic() {
        let html = r#"
            <table>
              <tr><th>Name</th><th>Score</th></tr>
              <tr><td>Alice</td><td>95</td></tr>
              <tr><td>Bob</td><td>87</td></tr>
            </table>
        "#;
        let tables = extract_tables_str(html);
        assert_eq!(tables.len(), 1);
        let md = &tables[0];
        assert!(md.contains("Name"), "header missing: {md}");
        assert!(md.contains("Score"), "header missing: {md}");
        assert!(md.contains("Alice"), "data missing: {md}");
        assert!(md.contains("---|"), "separator missing: {md}");
    }

    #[test]
    fn test_extract_tables_multiple() {
        let html = r#"
            <table><tr><td>T1</td></tr></table>
            <p>between</p>
            <table><tr><td>T2</td></tr></table>
        "#;
        let tables = extract_tables_str(html);
        assert_eq!(tables.len(), 2);
        assert!(tables[0].contains("T1"));
        assert!(tables[1].contains("T2"));
    }

    #[test]
    fn test_extract_tables_nested_outer_only() {
        let html = r#"
            <table>
              <tr><td>outer<table><tr><td>inner</td></tr></table></td></tr>
            </table>
        "#;
        let tables = extract_tables_str(html);
        assert_eq!(tables.len(), 1);
    }

    #[test]
    fn test_extract_tables_empty_doc() {
        assert!(extract_tables_str("<p>no tables</p>").is_empty());
    }
}
