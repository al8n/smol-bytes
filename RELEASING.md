# Releasing smol-bytes

This document describes the release process for 0.2 and later. It is a
checklist, not evidence that a release has already occurred.

The root `[workspace.package]` version in `Cargo.toml` is the source of truth
for all Cargo members and Python's dynamic version. `js/package.json` and both
root identities in `js/package-lock.json` must match it. The current published
npm baseline is `smol-bytes@0.1.8`; the next release tag is `v0.2.0`.

Pushing a tag named `v<version>` drives publication for all three registries:
crates.io, PyPI, and npm. No local command in this document publishes a
release.

## Trusted publishing

All registries use GitHub Actions OIDC trusted publishing from protected GitHub
environments:

| Registry | Workflow | Protected environment |
| --- | --- | --- |
| crates.io | `.github/workflows/crates.yml` | `crates-io` |
| PyPI | `.github/workflows/python.yml` | `pypi` |
| npm | `.github/workflows/wasm.yml` | `npm` |

The environments gate the publish jobs. Tag-triggered workflows obtain their
short-lived OIDC identity through GitHub Actions and publish the artifacts they
build or validate; do not substitute a manual credential flow.

## Pre-tag checklist

Run these commands from the release candidate commit. Use a clean worktree for
the final tag.

```bash
python3 scripts/check-release-version.py --tag v0.2.0 --check-js
python3 scripts/test_check_release_version.py

cargo fmt --all -- --check
cargo check --workspace --all-features
cargo test --workspace --all-features
cargo clippy --workspace --all-features --all-targets -- -D warnings
RUSTDOCFLAGS=-Dwarnings cargo doc -p smol-bytes --no-deps --all-features
cargo package -p smol-bytes
cargo publish -p smol-bytes --dry-run
```

Build and inspect the Python wheel from a fresh environment. Its version is
read dynamically from Cargo:

```bash
python -m venv .venv
source .venv/bin/activate
python -m pip install maturin pytest mypy
maturin build --release --features pyo3 --manifest-path smol-bytes-py/Cargo.toml
python -m pytest tests/python -v
python -m mypy --strict tests/python/typecheck_phase3.py
```

Build and validate the JavaScript package, including its generated Wasm output
and packed consumer:

```bash
rustup target add wasm32-unknown-unknown
cd js
npm ci
npm run build
npm test
npm pack --dry-run
```

Review the migration guide within `CHANGELOG.md` and the release-facing README
examples. Confirm the candidate has no unrelated changes, then create and push
the exact version tag:

```bash
git tag -a v0.2.0 -m "v0.2.0"
git push origin v0.2.0
```

## After the tag

Watch the tag workflows through their protected-environment approvals. After
they complete, verify the published metadata and installability from each
registry:

```bash
cargo search smol-bytes
python -m pip index versions smol-bytes
npm view smol-bytes version
```

Confirm the reported version is `0.2.0`, inspect the release artifacts and
workflow logs, and run a minimal downstream install for each binding surface.
If a workflow does not publish, investigate its artifact validation or
environment approval rather than retagging a different version.
