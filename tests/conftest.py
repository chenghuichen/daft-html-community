from __future__ import annotations

import pytest

import daft
import daft_html


@pytest.fixture(scope="session")
def sess():
    s = daft.Session()
    s.load_extension(daft_html)
    with s:
        yield s
