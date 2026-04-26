# daft-html

A native Daft extension for HTML processing. Built on [scraper](https://crates.io/crates/scraper) / [html5ever](https://crates.io/crates/html5ever), so it handles real-world malformed HTML correctly.

## Installation

```bash
pip install daft-html
```

Requires Python ≥ 3.10 and `daft ≥ 0.4`.

## Quick start

```python
import daft
import daft_html
from daft_html import html_to_text, html_get_title, html_extract_links
from daft import col

sess = daft.Session()
sess.load_extension(daft_html)

with sess:
    df = daft.from_pydict({
        "html": [
            "<html><head><title>Hello</title></head><body><p>World</p></body></html>",
            "<p>Just <b>text</b> here. <a href='https://example.com'>link</a></p>",
        ]
    })

    result = df.select(
        html_to_text(col("html")).alias("text"),
        html_get_title(col("html")).alias("title"),
        html_extract_links(col("html")).alias("links"),
    ).collect()

    # +-----------------+---------+---------------------------+
    # | text            | title   | links                     |
    # +-----------------+---------+---------------------------+
    # | World           | Hello   | []                        |
    # | Just text here. | None    | [https://example.com]     |
    # +-----------------+---------+---------------------------+
```

## Operators

### Document-level

| Function | Signature | Description |
|---|---|---|
| `html_to_text(expr)` | String → String | Extract plain text, discard all tags |
| `html_get_title(expr)` | String → String | Extract `<title>` text |
| `html_text_ratio(expr)` | String → Float64 | Ratio of visible text chars to raw HTML bytes |
| `html_extract_meta(expr, name)` | (String, str) → String | `<meta name\|property="…">` content value |
| `html_extract_links(expr)` | String → List[String] | All `<a href="…">` URLs |
| `html_extract_tables(expr)` | String → List[String] | `<table>` elements as Markdown strings |

### CSS-selector

| Function | Signature | Description |
|---|---|---|
| `html_extract_text(expr, selector)` | (String, str) → String | Inner text of first matching element |
| `html_get_attribute(expr, selector, attr)` | (String, str, str) → String | Attribute value of first matching element |
| `html_has_element(expr, selector)` | (String, str) → Bool | True if selector matches at least one element |
| `html_count_elements(expr, selector)` | (String, str) → Int64 | Count of elements matching selector |

## Development

Requires [uv](https://docs.astral.sh/uv/) and a Rust nightly toolchain (see `rust-toolchain.toml`).

```bash
make build   # cargo build (debug) + pip install -e .
make test    # pytest tests/ -v
```

## License

Apache-2.0
