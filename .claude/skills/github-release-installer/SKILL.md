---
name: github-release-installer
description: Set up a "push a git tag → GitHub Actions builds a single release artifact → users install with one PowerShell line" pipeline for a local-first app. Use when the user wants to add a release/distribution workflow, a one-line installer, or asks how to let users run the app without installing build toolchains.
---

# github-release-installer

This skill sets up the full **tag → CI → Release → one-line install** pipeline
for a local-first app, so end users install a prebuilt binary via a single
PowerShell command and never need the build toolchain.

## When to use

Trigger when the user asks for any of:
- "let users install/run without `make run` / building themselves"
- "one-line install like `irm ... | iex`"
- "release / distribution / publishing workflow"
- "CI that builds on tag push"
- "GitHub Actions release pipeline"

## The pipeline (what you build)

```
git tag vX.Y.Z && git push origin vX.Y.Z
  → GitHub Actions triggers on tags: v*
  → CI compiles a single self-contained artifact (frontend embedded if web UI)
  → CI creates a Release and uploads the artifact as an asset
  → installer/get.ps1 calls /releases/latest, downloads the asset, installs, launches
  → user runs: irm <raw get.ps1 url> | iex
```

## Prerequisites to verify BEFORE writing anything

1. **Repo is public** — GitHub returns 404 for raw URLs and unauthenticated API
   on private repos. Verify with an unauthenticated curl to the API:
   `curl -s https://api.github.com/repos/<owner>/<repo> | grep private`. If it's
   private, tell the user this pipeline requires public (or a token-based variant).
2. **Account is not flagged/restricted** — a flagged account's public repos also
   return 404 to anonymous access. Verify: open `https://github.com/<owner>` in
   an incognito window; if 404, the account is restricted. No code change fixes
   this — the user must contact GitHub support.
3. **The app can produce a single artifact** — Rust exe, Go binary, or a
   bundled single-file app. Multi-file outputs must be zipped. The whole point
   is one downloadable file. If the app requires a runtime the user doesn't have
   (e.g. Node, Python, JRE), this pipeline isn't the right fit — you'd be shipping
   a source app, not a self-contained binary.

## Files to create

### 1. `.github/workflows/release.yml`

Key elements (adapt build steps to the project's actual toolchain):

```yaml
name: release
on:
  push:
    tags:
      - 'v*'
permissions:
  contents: write   # required to create Releases
jobs:
  build-release:
    runs-on: windows-latest   # match the users' OS; use ubuntu-latest for cross-platform binaries
    steps:
      - uses: actions/checkout@v4
      # ... install toolchain (Rust/Node/Go/etc.) ...
      # ... build the single artifact ...
      - name: Stage release asset
        shell: pwsh
        run: Copy-Item <built-artifact-path> <fixed-asset-name> -Force
      - uses: softprops/action-gh-release@v2
        with:
          files: <fixed-asset-name>
          generate_release_notes: true
          prerelease: ${{ contains(github.ref_name, '-') }}
```

Critical details:
- **trigger is `tags: v*`**, NOT branch push. Pushing `main` must NOT trigger a
  release build (avoids noise; releases are deliberate version points).
- **`permissions: contents: write`** at job/workflow level — without it, the
  Release-creation step fails with 403.
- **asset name is fixed** (e.g. `glucose-dashboard.exe`) — the installer script
  looks up the asset by this exact name. The build artifact path may differ;
  the "Stage" step renames it to the fixed name.
- **`prerelease`** logic: tag names containing `-` (e.g. `v1.0.0-beta`) are
  marked prerelease so `/releases/latest` won't return them (installer grabs
  latest non-prerelease only).

### 2. `installer/get.ps1`

The one-line installer. Core logic (adapt `$ExeName`, `$InstallRoot`):

```powershell
[CmdletBinding()]
param([string]$Repo = '<owner>/<repo>', [switch]$NoLaunch)
$ErrorActionPreference = 'Stop'
$InstallRoot = Join-Path $env:LOCALAPPDATA '<AppName>'
$ExeName = '<app>.exe'
$CmdName = '<app>.cmd'

# 1. Query latest release metadata
$release = Invoke-RestMethod "https://api.github.com/repos/$Repo/releases/latest" `
    -Headers @{ 'User-Agent' = 'installer' }
$asset = $release.assets | Where-Object { $_.name -eq $ExeName } | Select-Object -First 1
if (-not $asset) { throw "asset $ExeName not found in latest release" }

# 2. Download the asset
Invoke-WebRequest -Uri $asset.browser_download_url -OutFile <tmp> -UseBasicParsing
Move-Item <tmp> (Join-Path $InstallRoot $ExeName) -Force

# 3. Create .cmd shim + add to User PATH (idempotent)
# 4. Launch (unless -NoLaunch): Start-Process the exe
```

Critical details:
- **never overwrite the config file** — if the app persists user config
  (e.g. `.glucose-dashboard.json`), check `Test-Path` before writing and skip.
- **PATH update is idempotent** — check if `$InstallRoot` already in
  `[Environment]::GetEnvironmentVariable('Path','User')` before appending.
- **PATH only affects NEW terminals** — tell the user in output.
- **`irm | iex` cannot pass params** — document the dot-source alternative
  `& (irm <url>) -NoLaunch` for users who need switches.

### 3. Self-contained binary (the architectural enabler)

For a web-UI app, the single artifact only works if the frontend is **embedded
into the binary** (not served from a disk `frontend/dist` relative to CWD).
Otherwise the installed binary breaks when launched from a different working
directory.

- **Rust + web frontend**: use `rust-embed` to embed `frontend/dist` at compile
  time. Fallback handler serves embedded assets; dev mode can still read from
  disk when `frontend/dist` exists (keeps dev workflow intact).
- Also make config path resolve relative to the **exe location** (not CWD), so
  launching via a PATH shim from any directory finds/creates config next to the
  exe. Resolution order: env override → CWD config file if exists → exe dir.

If the app already reads assets from disk relative to CWD, that MUST be fixed
first or the installed binary will 404 its own frontend.

## Verification (no screenshots — CI runs the real build)

1. **CI triggered**: `curl -s https://api.github.com/repos/<owner>/<repo>/actions/runs?per_page=3`
   shows a run with `event: push`, `head_branch: v<tag>`, `status: completed`,
   `conclusion: success`.
2. **Release created**: `curl -s https://api.github.com/repos/<owner>/<repo>/releases`
   shows the tag with an asset whose `name` matches the fixed asset name and a
   non-zero `size`.
3. **Raw installer URL is 200**: `curl -sI https://raw.githubusercontent.com/<owner>/<repo>/main/installer/get.ps1`
   returns 200 (confirms public + not-flagged + correct path).
4. **Binary is self-contained**: build the release artifact locally, copy it to
   a clean temp dir with NO `frontend/dist`, launch it, and curl the root path
   — it must return 200 with the frontend HTML, proving assets are embedded and
   not disk-dependent.

## Release steps (tell the user to run)

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

Then the user installs on Windows with:

```powershell
irm https://raw.githubusercontent.com/<owner>/<repo>/main/installer/get.ps1 | iex
```

## Failure-mode checklist (when something returns 404)

1. Repo private? → `curl api.../repos/<o>/<r>` → `private: true`. Fix: make public.
2. Account flagged? → incognito `https://github.com/<owner>` returns 404 even
   though owner sees it fine logged-in. Fix: contact GitHub support (no code fix).
3. Wrong owner/repo name or case? → `git remote -v` shows the real SSH path;
   compare to the raw URL owner/repo.
4. Workflow didn't trigger? → confirm trigger is `tags: v*` and the pushed tag
   starts with `v`; confirm Actions enabled in repo Settings → Actions.
5. Release exists but no asset? → check `permissions: contents: write` and the
   `files:` path in the workflow; check the "Stage release asset" step actually
   produced the file.

## Reference

For the worked example this skill is derived from, see `docs/Github_CI.md` in
this repo — it walks through the GlucoseDashboard pipeline end to end with the
reasoning behind each design choice.