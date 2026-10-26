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


@pytest.fixture
def published(monkeypatch):
    candidate = {
        "id": 1,
        "tag_name": "v1.0.0",
        "draft": False,
        "published_at": "2026-01-03T00:00:00Z",
    }
    monkeypatch.setattr(
        MODULE,
        "release",
        lambda tag: candidate if tag == candidate["tag_name"] else None,
    )
    return candidate


def test_dispatch_exact_tag(published):
    assert (
        MODULE.candidate_for_event("workflow_dispatch", {"inputs": {"tag": "v1.0.0"}})
        == published
    )


def test_dispatch_rejects_draft(published):
    published["draft"] = True
    with pytest.raises(ValueError, match="published release"):
        MODULE.candidate_for_event("workflow_dispatch", {"inputs": {"tag": "v1.0.0"}})


@pytest.mark.parametrize(
    ("field", "value"),
    [
        ("conclusion", "failure"),
        ("event", "pull_request"),
        ("head_repository", {"full_name": "other/repo"}),
        ("name", "CI"),
    ],
)
def test_untrusted_workflow(field, value):
    run = {
        "conclusion": "success",
        "event": "push",
        "head_repository": {"full_name": MODULE.SOURCE},
        "name": "Development release",
    }
    run[field] = value
    with pytest.raises(ValueError, match="Untrusted"):
        MODULE.candidate_for_event("workflow_run", {"workflow_run": run})


def test_release_identity_mismatch(published):
    with pytest.raises(ValueError, match="no longer matches"):
        MODULE.candidate_for_event(
            "release", {"release": {"tag_name": "v1.0.0", "id": 2}}
        )


@pytest.mark.parametrize("publish_status", ["success", "skipped", "failure"])
def test_completed_workflow(monkeypatch, published, publish_status):
    run = {
        "id": 123,
        "run_attempt": 2,
        "conclusion": "success",
        "event": "push",
        "head_repository": {"full_name": MODULE.SOURCE},
        "name": "Development release",
        "head_branch": "v1.0.0",
        "head_sha": "a" * 40,
    }

    def fake_api(path):
        if "/jobs?" in path:
            assert "/attempts/2/" in path
            return {
                "jobs": [
                    {
                        "name": "Publish development prerelease",
                        "conclusion": publish_status,
                    }
                ]
            }
        return {"sha": "a" * 40}

    monkeypatch.setattr(MODULE, "api", fake_api)
    result = MODULE.candidate_for_event("workflow_run", {"workflow_run": run})
    assert result == (published if publish_status == "success" else None)
    if publish_status == "success":
        run["head_sha"] = "b" * 40
        with pytest.raises(ValueError, match="does not match"):
            MODULE.candidate_for_event("workflow_run", {"workflow_run": run})


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
