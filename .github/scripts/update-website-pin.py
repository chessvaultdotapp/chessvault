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


def main():
    with open(os.environ["GITHUB_EVENT_PATH"], encoding="utf-8") as stream:
        event_release = json.load(stream)["release"]
    candidate = release(event_release["tag_name"])
    if candidate["id"] != event_release["id"] or candidate["draft"]:
        raise ValueError("Triggering release no longer matches a published release")
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
