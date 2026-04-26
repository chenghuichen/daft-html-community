"""daft-html: Daft native extension for HTML processing.

Provides HTML operators (parsing, extraction, transformation) implemented in
Rust via daft-ext, exposed as first-class Daft expression functions.

All operators use a proper HTML5 DOM parser (scraper / html5ever) so they
handle malformed HTML, implicit element insertion, and foster parenting
correctly.

Load the extension into a Daft session before calling any operator::

    import daft
    import daft_html

    sess = daft.Session()
    sess.load_extension(daft_html)
    with sess:
        df.select(html_to_text(col("html"))).collect()

Document-level extraction
--------------------------
html_to_text(expr)
    Extract plain text from HTML, discarding all tags.

html_get_title(expr)
    Extract the ``<title>`` text of an HTML document.

html_text_ratio(expr)
    Ratio of visible text characters to total HTML length (Float64).
    Useful as a content-quality signal: low values indicate JS-heavy or
    ad-heavy pages with little readable content.

html_extract_meta(expr, name)
    Return the ``content`` attribute of the first ``<meta>`` tag whose
    ``name`` or ``property`` attribute matches ``name``.
    Supports Open Graph (``"og:title"``, ``"og:description"``, …) and
    standard meta tags (``"description"``, ``"keywords"``, …).

html_extract_links(expr)
    Return all ``href`` values from ``<a>`` elements as ``List[String]``.

html_extract_tables(expr)
    Return all ``<table>`` elements rendered as Markdown strings,
    as ``List[String]``.  Header rows (``<th>``) produce the ``|---|``
    separator line.  Only top-level tables are captured; nested tables
    are skipped.

CSS-selector operators
-----------------------
html_extract_text(expr, selector)
    Return the inner text of the first element matching the CSS selector.
    Returns ``null`` when no element matches.

html_get_attribute(expr, selector, attr)
    Return the value of ``attr`` on the first element matching the CSS
    selector.  Returns ``null`` when no element matches or the attribute
    is absent.

html_has_element(expr, selector)
    Return ``True`` if the document contains at least one element matching
    the CSS selector.

html_count_elements(expr, selector)
    Return the count (Int64) of elements matching the CSS selector.
"""

from __future__ import annotations

from pathlib import Path
from typing import TYPE_CHECKING

if TYPE_CHECKING:
    from daft.expressions import Expression


def _lib_path() -> Path:
    """Return the path to the compiled native library shipped with this package."""
    pkg_dir = Path(__file__).parent
    for ext in (".so", ".dylib", ".dll"):
        candidates = list(pkg_dir.glob(f"*{ext}"))
        if candidates:
            return candidates[0]
    raise FileNotFoundError(
        f"No native library found in {pkg_dir}. "
        "If you installed via pip, the wheel may not include a binary for your "
        "platform. If developing locally, run `make build` to compile."
    )


# ── Document-level extraction ─────────────────────────────────────────────────


def html_to_text(expr: Expression) -> Expression:
    """Strip HTML tags from a string column, returning plain text.

    Content inside ``<script>``, ``<style>``, ``<head>``, ``<noscript>``,
    and ``<template>`` is discarded entirely.  Block-level elements inject a
    newline separator between their text runs.

    Args:
        expr: A String expression containing HTML text.

    Returns:
        Expression: Plain text with all HTML tags removed.
    """
    import daft

    return daft.get_function("html_to_text", expr)


def html_get_title(expr: Expression) -> Expression:
    """Extract the page title from the ``<title>`` element.

    Returns ``null`` when no ``<title>`` tag is present or it is empty.

    Args:
        expr: A String expression containing HTML text.

    Returns:
        Expression: The trimmed title string, or null.
    """
    import daft

    return daft.get_function("html_get_title", expr)


def html_text_ratio(expr: Expression) -> Expression:
    """Compute the ratio of visible text characters to total HTML length.

    Counts non-whitespace characters in visible text (content inside
    ``<script>``, ``<style>``, ``<head>``, ``<noscript>``, and
    ``<template>`` is excluded), divided by the raw byte length of the HTML
    string.  Returns a Float64 in ``[0, 1]``.

    Useful as a content-quality signal: pages with a ratio below ~0.1 are
    typically JavaScript-heavy SPA shells or ad-dominated pages.

    Args:
        expr: A String expression containing HTML text.

    Returns:
        Expression: Float64 ratio, or null for null input.
    """
    import daft

    return daft.get_function("html_text_ratio", expr)


def html_extract_meta(expr: Expression, name: str) -> Expression:
    """Extract the ``content`` of a ``<meta>`` tag by name or property.

    Matches the first ``<meta>`` whose ``name`` **or** ``property`` attribute
    equals ``name`` (case-sensitive), and returns its ``content`` attribute.
    Returns ``null`` when no matching tag is found.

    Supports both standard meta tags and Open Graph properties::

        html_extract_meta(col("html"), "description")
        html_extract_meta(col("html"), "og:title")
        html_extract_meta(col("html"), "og:image")

    Args:
        expr: A String expression containing HTML text.
        name: The ``name`` or ``property`` attribute value to match.

    Returns:
        Expression: The ``content`` string, or null.
    """
    import daft

    return daft.get_function("html_extract_meta", expr, daft.lit(name))


def html_extract_links(expr: Expression) -> Expression:
    """Extract all hyperlink URLs from ``<a href="…">`` elements.

    Returns a ``List[String]`` column.  URLs are returned as-is (no
    normalisation or deduplication).  Use ``.explode()`` on the result
    to produce one URL per row.

    Args:
        expr: A String expression containing HTML text.

    Returns:
        Expression: List[String] of href values, or null for null input.
    """
    import daft

    return daft.get_function("html_extract_links", expr)


def html_extract_tables(expr: Expression) -> Expression:
    """Extract ``<table>`` elements and render them as Markdown strings.

    Returns a ``List[String]`` column where each element is one table
    rendered in GitHub-flavoured Markdown.  Rows containing ``<th>`` cells
    are treated as header rows and produce the ``|---|`` separator line.
    Only top-level tables are captured; nested tables are skipped.

    Use ``.explode()`` on the result to produce one Markdown table per row.

    Args:
        expr: A String expression containing HTML text.

    Returns:
        Expression: List[String] of Markdown table strings, or null for null
        input.
    """
    import daft

    return daft.get_function("html_extract_tables", expr)


# ── CSS-selector operators ────────────────────────────────────────────────────


def html_extract_text(expr: Expression, selector: str) -> Expression:
    """Extract the inner text of the first element matching a CSS selector.

    The selector is compiled once per call and applied to every row.
    Returns ``null`` when no element matches or the matched element has no
    text content.

    Example::

        html_extract_text(col("html"), "h1")
        html_extract_text(col("html"), "div.article-body > p:first-child")

    Args:
        expr: A String expression containing HTML text.
        selector: A CSS selector string (e.g. ``"h1"``, ``"#main p"``).

    Returns:
        Expression: Trimmed inner text string, or null.
    """
    import daft

    return daft.get_function("html_extract_text", expr, daft.lit(selector))


def html_get_attribute(expr: Expression, selector: str, attr: str) -> Expression:
    """Return an attribute value from the first element matching a CSS selector.

    Returns ``null`` when no element matches or the element does not have the
    requested attribute.

    Example::

        html_get_attribute(col("html"), "img.hero", "src")
        html_get_attribute(col("html"), "link[rel=canonical]", "href")

    Args:
        expr: A String expression containing HTML text.
        selector: A CSS selector string.
        attr: The attribute name to retrieve (e.g. ``"src"``, ``"href"``).

    Returns:
        Expression: Attribute value string, or null.
    """
    import daft

    return daft.get_function("html_get_attribute", expr, daft.lit(selector), daft.lit(attr))


def html_has_element(expr: Expression, selector: str) -> Expression:
    """Return ``True`` if the document contains at least one matching element.

    Useful as a boolean filter or feature flag for downstream processing.

    Example::

        html_has_element(col("html"), "table")
        html_has_element(col("html"), "[data-paywall]")

    Args:
        expr: A String expression containing HTML text.
        selector: A CSS selector string.

    Returns:
        Expression: Boolean, or null for null input.
    """
    import daft

    return daft.get_function("html_has_element", expr, daft.lit(selector))


def html_count_elements(expr: Expression, selector: str) -> Expression:
    """Count the number of elements matching a CSS selector.

    Returns an ``Int64`` column.  Returns ``0`` (not null) when no elements
    match; returns ``null`` only for null input.

    Example::

        html_count_elements(col("html"), "img")
        html_count_elements(col("html"), "a[href]")

    Args:
        expr: A String expression containing HTML text.
        selector: A CSS selector string.

    Returns:
        Expression: Int64 count, or null for null input.
    """
    import daft

    return daft.get_function("html_count_elements", expr, daft.lit(selector))


__all__ = [
    "html_to_text",
    "html_get_title",
    "html_text_ratio",
    "html_extract_meta",
    "html_extract_links",
    "html_extract_tables",
    "html_extract_text",
    "html_get_attribute",
    "html_has_element",
    "html_count_elements",
]
