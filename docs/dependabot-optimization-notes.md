# Dependabot Optimization Notes

## What changed

1. **`.github/dependabot.yml`**
   - Changed cargo updates from `daily` to `weekly` (same cadence as github-actions).
   - Added explicit `day: monday`, `time: "04:00"`, `timezone: Asia/Shanghai`, and `target-branch: master` to both ecosystems.
   - Updated commit-message prefixes to conventional-commit style: `build(deps)` for cargo, `ci` for github-actions.
   - Removed `include: "scope"` to avoid duplicated scopes like `build(deps)(deps):`.
   - Replaced invalid `XenoAmess-bot` reviewer/assignee with `XenoAmess`.
   - Added labels (`dependencies` + ecosystem label) and tightened `open-pull-requests-limit` (5 for actions, 10 for cargo).
   - Kept one-PR-per-dependency behavior (no `groups:` block).

2. **`.github/workflows/auto-merge.yml`** (new)
   - Enables auto-merge for Dependabot patch/minor PRs.
   - Also auto-merges semver-major GitHub Actions bumps (usually safe runtime bumps).
   - Leaves semver-major Rust/cargo bumps for human review.
   - Uses `pull_request_target` so the workflow can access the `MYTOKEN` secret on Dependabot PRs.
   - Matches both Dependabot login forms: `dependabot[bot]` and `app/dependabot`.

3. **Removed `.github/workflows/dependabot-auto-merge.yml`**
   - It was a duplicate, older workflow that only matched the legacy `dependabot[bot]` login and used `GITHUB_TOKEN` for the merge step, which fails on workflow-file PRs.

4. **Repository settings**
   - Enabled `allow_auto_merge`.
   - Configured branch protection on `master` with all CI checks required and `strict: true`.
   - Set `MYTOKEN` repository secret to the maintainer's OAuth token (admin access).

5. **Remote URL**
   - Updated local `origin` from the stale `XenoAmess/untar` (redirects to a deleted repo) to the actual `XenoAmess-Auto/untar`.
   - Switched push URL to SSH to avoid the OAuth `workflow` scope restriction when pushing workflow files.

## Key finding

Dependabot PRs triggered by the `pull_request` event do **not** have access to repository secrets, so `secrets.MYTOKEN` resolves to an empty value. The auto-merge workflow must use `pull_request_target` to read the PAT secret.

## Verification

- PR #27 (`ci: bump dependabot/fetch-metadata from 2 to 3`) was approved and auto-merged via the new workflow after all required checks passed.

## Remaining manual follow-up

- The README files still contain badge/clone URLs pointing to the old `XenoAmess-bot/untar` location. Update them when convenient.
