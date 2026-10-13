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

The crates.io trusted publisher is configured for owner `al8n`, repository
`smol-bytes`, workflow `.github/workflows/crates.yml`, and environment
`crates-io`. The PyPI pending publisher is configured for owner `al8n`,
repository `smol-bytes`, workflow `.github/workflows/python.yml`, and environment
`pypi`.

The npm package `smol-bytes` does not exist yet, and this repository has no
`NPM_TOKEN` secret, so npm trusted publishing cannot bootstrap the package.
After synchronizing all versions and building and testing the package, the npm
account owner must perform the one-time initial publish manually while
authenticated with 2FA. Do not perform that publication from an automated
workflow.

After the package exists, configure its npm package Settings trusted publisher
with GitHub owner `al8n`, repository `smol-bytes`, workflow filename exactly
`wasm.yml`, environment `npm`, and allowed action direct `npm publish`. Complete
the first subsequent OIDC publication within 2 days (48 hours) after creating
the trusted publisher, as required by current npm policy. If the configuration
expires, delete and recreate it before retrying. If a temporary credential was
used for the initial publication, remove or revoke it only after the OIDC
publication succeeds.
