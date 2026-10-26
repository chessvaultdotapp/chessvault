# Website release pin automation

`.github/workflows/website-release-pin.yml` handles published stable releases and prereleases.
Draft creation does not trigger it. The workflow reads trusted automation from `main`, resolves the triggering
release tag to a full commit SHA, and updates `changelogs.json` in `chessvaultdotapp/website`.
It never writes application Git refs or changes release tags.

Releases published by the built-in `GITHUB_TOKEN` do not trigger other release workflows.
A `workflow_run` completion trigger therefore also handles Development release and Stable release.
It accepts only successful push runs from this repository, checks that the publication job succeeded in that
specific run attempt, and resolves the run's exact tag. The tag commit must match the run's head SHA.
Skipped publication jobs are ignored. No artifacts or code from the triggering run are executed;
automation is always checked out from `main`.

For recovery or verification of an existing release, run **Website release pin** from the Actions tab using
**Run workflow** on `main` and enter its exact published tag. Alternatively:

```sh
gh workflow run website-release-pin.yml --ref main -f 'tag=v0.1.0+26w44a'
```

All three trigger paths share the same ordering, duplicate-PR, and draft guards.

## Credentials

Create a fine-grained PAT owned by an account with access to `chessvaultdotapp/website`:

- Limit repository access to `chessvaultdotapp/website`.
- Grant Contents: read and write, and Pull requests: read and write (Metadata read is implicit).
- Store it as the **application repository** Actions secret `WEBSITE_PIN_TOKEN`.
- Obtain organization approval if required, and rotate the token before its expiration.

The source repository is public, so source release and commit reads do not need additional permissions.
The workflow's built-in token has only contents-read permission; it is not used for website writes.
PAT-created PRs and commits trigger website checks, unlike writes using the website's own built-in Actions token.
Do not configure automatic merging or bypass website branch protection for this account.
Require website checks before merging. Merging deploys the updated changelogs through the website workflow.

## Ordering and retries

One reusable branch, `automation/changelog-release-pin`, holds the pending update.
An existing open PR is updated rather than duplicated. Keep this branch reserved for automation.
Publication timestamps, not SemVer or a moving latest-release reference, order releases across both channels.
Candidates must be strictly newer than the merged pin and must not precede the pending pin.
Equal timestamps for different tags are skipped conservatively. Missing releases or invalid pins fail the job
rather than guessing. Republishing an older tag with a newer publication timestamp counts as a new publication.

Workflow concurrency serializes writes without cancelling an in-progress update. GitHub can replace queued runs;
timestamp guards prevent an older surviving run from overwriting a newer merged or pending pin.
Rerunning a failed release job repairs an interrupted update, including a commit written before PR creation failed.
Existing release tags are never moved. Avoid manually editing the automation branch during a run.

## Verification

Run offline ordering tests with:

```sh
venvs/tag-tool/bin/python -m pytest .github/scripts/test_website_pin.py -q
```

After configuring the secret, publish a release or rerun its published-event job. Verify the website PR records
that release's tag and exact commit SHA, changes only `changelogs.json`, and triggers passing website checks.
Repeat for the other release channel. Rerun the job to confirm no duplicate PR, then rerun an older release job
to confirm it cannot replace the newer pending or merged pin. Draft saves must not create workflow runs.
These live checks require repository credentials and are not performed by the offline tests.
