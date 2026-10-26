"""Offline publication-order regression tests for website pin automation."""

import importlib.util
from pathlib import Path

import pytest

SPEC = importlib.util.spec_from_file_location(
    "website_pin", Path(__file__).with_name("update-website-pin.py")
)
assert SPEC and SPEC.loader
MODULE = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(MODULE)


@pytest.mark.parametrize(
    ("timestamp", "draft", "expected"),
    [
        ("2026-01-03T00:00:00Z", False, True),
        ("2026-01-02T00:00:00Z", False, False),
        ("2026-01-01T00:00:00Z", False, False),
        ("2026-01-03T00:00:00Z", True, False),
        (None, True, False),
    ],
)
def test_publication_order(timestamp, draft, expected):
    current = {"published_at": "2026-01-02T00:00:00Z", "draft": False}
    assert (
        MODULE.is_newer({"published_at": timestamp, "draft": draft}, current)
        is expected
    )
