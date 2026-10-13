# Releasing smol-bytes

The version in `[workspace.package]` in the root `Cargo.toml` is the source of
truth for all three Cargo packages and the Python package. Before pushing a
release tag, set `js/package.json` to the same version and use the exact tag
`v<version>`. Run
`python3 scripts/check-release-version.py --tag v<version> --check-js` to check
that metadata locally. `Cargo.lock` is intentionally not tracked in this
repository.

Pushed `v*` tags run the registry publishing jobs. Pull requests receive no
registry credentials, and manual dispatch of the crates.io workflow performs
verification only.

## GitHub environments

The GitHub environments `crates-io`, `pypi`, and `npm` exist and require
approval from `al8n`.

## Trusted publishers

The crates.io trusted publisher is configured with owner `al8n`, repository
`smol-bytes`, workflow `.github/workflows/crates.yml`, and environment
`crates-io`. The PyPI trusted publisher is configured with owner `al8n`,
repository `smol-bytes`, workflow `.github/workflows/python.yml`, and
environment `pypi`.

The npm trusted publisher for package `smol-bytes` is configured with GitHub
owner `al8n`, repository `smol-bytes`, workflow `.github/workflows/wasm.yml`,
and environment `npm`; its allowed action is direct `npm publish`. The npm
package `smol-bytes@0.1.7` is already published, and this repository has no
`NPM_TOKEN` secret. npm releases use OIDC trusted publishing.

Whenever an npm trusted publisher is newly created or recreated, its first OIDC
publish must complete within 48 hours, as required by current npm policy. If
the configuration expires, delete and recreate it before retrying.
