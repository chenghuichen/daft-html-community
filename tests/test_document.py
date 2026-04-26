from __future__ import annotations

import daft
from daft import col

from daft_html import (
    html_extract_links,
    html_extract_meta,
    html_extract_tables,
    html_get_title,
    html_text_ratio,
    html_to_text,
)


def _collect(df: daft.DataFrame, column: str) -> list:
    return df.collect().to_pydict()[column]


def test_html_to_text(sess):
    df = daft.from_pydict({"html": ["<p>Hello</p>", None]})
    result = _collect(df.select(html_to_text(col("html")).alias("out")), "out")
    assert result == ["Hello", None]


def test_html_get_title(sess):
    df = daft.from_pydict({"html": ["<title>My Page</title>", "<p>none</p>", None]})
    result = _collect(df.select(html_get_title(col("html")).alias("out")), "out")
    assert result == ["My Page", None, None]


def test_html_text_ratio(sess):
    df = daft.from_pydict({"html": ["<p>hello</p>", None]})
    result = _collect(df.select(html_text_ratio(col("html")).alias("out")), "out")
    assert result[0] > 0.2
    assert result[1] is None


def test_html_extract_meta(sess):
    html = '<meta name="description" content="A page">'
    df = daft.from_pydict({"html": [html, "<p>none</p>", None]})
    result = _collect(
        df.select(html_extract_meta(col("html"), "description").alias("out")), "out"
    )
    assert result == ["A page", None, None]


def test_html_extract_links(sess):
    html = '<a href="https://a.com">a</a><a href="/b">b</a>'
    df = daft.from_pydict({"html": [html, None]})
    result = _collect(df.select(html_extract_links(col("html")).alias("out")), "out")
    assert result[0] == ["https://a.com", "/b"]
    assert result[1] is None


def test_html_extract_tables(sess):
    html = "<table><tr><th>K</th></tr><tr><td>V</td></tr></table>"
    df = daft.from_pydict({"html": [html, None]})
    result = _collect(df.select(html_extract_tables(col("html")).alias("out")), "out")
    assert len(result[0]) == 1
    assert "K" in result[0][0]
    assert result[1] is None
