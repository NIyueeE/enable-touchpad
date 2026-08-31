# Release

> English | [简体中文](release.zh.md)

Cutting a release is a single tag push:

```bash
git tag v0.1.0
git push origin v0.1.0
```

`.github/workflows/release.yml` then creates the GitHub Release and builds the
binary for three targets, attaching the archives to the release:

| Runner | Target |
|--------|--------|
| ubuntu-latest | x86_64-unknown-linux-gnu |
| macos-latest | aarch64-apple-darwin |
| windows-latest | x86_64-pc-windows-msvc |

Each archive contains the binary plus `LICENSE`, `LICENSE-MIT`,
`LICENSE-APACHE`, and `README.md`.

## Test builds (not releases)

`.github/workflows/test-build.yml` builds **test artifacts** for chosen
platforms from any commit — dispatch it from the Actions tab
(**Test build → Run workflow**), pick a `ref` (commit SHA, branch, or tag)
and `targets` (`linux`, `macos`, `windows`). Artifacts are ephemeral
(7-day retention) and never published as a Release. Releases stay tag-driven.

## Notes

- **Release notes come from `CHANGELOG.md`** (Keep a Changelog format): record
  changes under `## [Unreleased]`, rename it to the version section before
  tagging. The workflow fails if the tag's section is missing.
- Keep `version` in `Cargo.toml` in sync with the tag before pushing it
  (manually, or automate with a tool like release-plz).
- The workflow needs `contents: write` — already declared in the file.
- Re-running a failed build: delete and re-push the tag
  (`git push origin :v0.1.0`), or re-run the workflow from the Actions tab.
