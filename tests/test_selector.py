from __future__ import annotations

import daft
from daft import col

from daft_html import (
    html_count_elements,
    html_extract_text,
    html_get_attribute,
    html_has_element,
)


def _collect(df: daft.DataFrame, column: str) -> list:
    return df.collect().to_pydict()[column]


def test_html_extract_text(sess):
    df = daft.from_pydict({"html": ["<h1>Title</h1>", "<p>no h1</p>", None]})
    result = _collect(
        df.select(html_extract_text(col("html"), "h1").alias("out")), "out"
    )
    assert result == ["Title", None, None]


def test_html_get_attribute(sess):
    df = daft.from_pydict({"html": ['<img src="a.jpg">', "<p>text</p>", None]})
    result = _collect(
        df.select(html_get_attribute(col("html"), "img", "src").alias("out")), "out"
    )
    assert result == ["a.jpg", None, None]


def test_html_has_element(sess):
    df = daft.from_pydict({"html": ["<table></table>", "<p>text</p>", None]})
    result = _collect(
        df.select(html_has_element(col("html"), "table").alias("out")), "out"
    )
    assert result == [True, False, None]


def test_html_count_elements(sess):
    df = daft.from_pydict({"html": ["<li>a</li><li>b</li><li>c</li>", "<p>x</p>", None]})
    result = _collect(
        df.select(html_count_elements(col("html"), "li").alias("out")), "out"
    )
    assert result == [3, 0, None]
