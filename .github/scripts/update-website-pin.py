"""Update the website pin using gh and a narrowly scoped website token."""

import base64
import json
import os
import subprocess
from datetime import datetime

SOURCE = "chessvaultdotapp/chessvault"
WEBSITE = "chessvaultdotapp/website"
BRANCH = "automation/changelog-release-pin"


def api(path, method="GET", **fields):
    command = ["gh", "api", path, "--method", method]
    for key, value in fields.items():
        command.extend(["-f", f"{key}={value}"])
    return json.loads(subprocess.check_output(command, text=True))


def release(tag):
    from urllib.parse import quote

    return api(f"repos/{SOURCE}/releases/tags/{quote(tag, safe='')}")


def is_newer(candidate, current):
    """Publication order includes prereleases; equal timestamps fail closed."""
    if candidate["draft"] or not candidate["published_at"]:
        return False
    return datetime.fromisoformat(candidate["published_at"]) > datetime.fromisoformat(
        current["published_at"]
    )


def pin_at(ref):
    from urllib.parse import quote

    data = api(f"repos/{WEBSITE}/contents/changelogs.json?ref={quote(ref, safe='')}")
    return json.loads(base64.b64decode(data["content"])), data["sha"]


def candidate_for_event(event_name, event):
    """Resolve only the exact release named by a trusted event, never latest."""
    if event_name == "release":
        candidate = release(event["release"]["tag_name"])
        if candidate["id"] != event["release"]["id"]:
            raise ValueError("Triggering release no longer matches")
    elif event_name == "workflow_dispatch":
        candidate = release(event["inputs"]["tag"])
    elif event_name == "workflow_run":
        run = event["workflow_run"]
        if (
            run["conclusion"] != "success"
            or run["event"] != "push"
            or run["head_repository"]["full_name"] != SOURCE
            or run["name"] not in {"Development release", "Stable release"}
        ):
            raise ValueError("Untrusted or unsuccessful release workflow")
        jobs = api(
            f"repos/{SOURCE}/actions/runs/{run['id']}/attempts/{run['run_attempt']}/jobs?per_page=100"
        )["jobs"]
        publish_name = (
            "Publish development prerelease"
            if run["name"] == "Development release"
            else "Publish stable release"
        )
        if not any(
            job["name"] == publish_name and job["conclusion"] == "success"
            for job in jobs
        ):
            print("Release publication job did not run successfully; nothing to pin")
            return None
        candidate = release(run["head_branch"])
        from urllib.parse import quote

        commit = api(f"repos/{SOURCE}/commits/{quote(candidate['tag_name'], safe='')}")[
            "sha"
        ]
        if commit != run["head_sha"]:
            raise ValueError("Release tag does not match the completed workflow commit")
    else:
        raise ValueError(f"Unsupported event: {event_name}")
    if candidate["draft"] or not candidate["published_at"]:
        raise ValueError("Expected a published release")
    return candidate


def main():
    with open(os.environ["GITHUB_EVENT_PATH"], encoding="utf-8") as stream:
        event = json.load(stream)
    candidate = candidate_for_event(os.environ["GITHUB_EVENT_NAME"], event)
    if candidate is None:
        return
    # Git's commit endpoint peels annotated tags to their commit object.
    from urllib.parse import quote

    commit = api(f"repos/{SOURCE}/commits/{quote(candidate['tag_name'], safe='')}")[
        "sha"
    ]
    if len(commit) != 40 or any(char not in "0123456789abcdef" for char in commit):
        raise ValueError("Expected a full commit SHA")
    default = api(f"repos/{WEBSITE}")["default_branch"]
    merged, _ = pin_at(default)
    if not is_newer(candidate, release(merged["tag"])):
        print("Website already pins this release or a newer publication")
        return
    branches = api(f"repos/{WEBSITE}/git/matching-refs/heads/{BRANCH}")
    exists = any(item["ref"] == f"refs/heads/{BRANCH}" for item in branches)
    if exists:
        pending, blob_sha = pin_at(BRANCH)
        if pending["tag"] != candidate["tag_name"] and not is_newer(
            candidate, release(pending["tag"])
        ):
            print("Pending website update is newer")
            return
    else:
        head = api(f"repos/{WEBSITE}/git/ref/heads/{default}")["object"]["sha"]
        api(f"repos/{WEBSITE}/git/refs", "POST", ref=f"refs/heads/{BRANCH}", sha=head)
        _, blob_sha = pin_at(BRANCH)
    pin = {"tag": candidate["tag_name"], "commit": commit}
    pending, _ = pin_at(BRANCH)
    if pending != pin:
        content = base64.b64encode((json.dumps(pin, indent=2) + "\n").encode()).decode()
        api(
            f"repos/{WEBSITE}/contents/changelogs.json",
            "PUT",
            message=f"Update changelogs to {pin['tag']}",
            content=content,
            sha=blob_sha,
            branch=BRANCH,
        )
    title = f"Update changelogs to {pin['tag']}"
    body = (
        f"Release: {candidate['html_url']}\n\nTag: `{pin['tag']}`\nCommit: `{commit}`\n"
    )
    pulls = api(f"repos/{WEBSITE}/pulls?head=chessvaultdotapp:{BRANCH}&state=open")
    if pulls:
        api(
            f"repos/{WEBSITE}/pulls/{pulls[0]['number']}",
            "PATCH",
            title=title,
            body=body,
        )
    else:
        api(
            f"repos/{WEBSITE}/pulls",
            "POST",
            title=title,
            body=body,
            head=BRANCH,
            base=default,
        )


if __name__ == "__main__":
    main()
