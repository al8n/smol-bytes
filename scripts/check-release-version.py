#!/usr/bin/env python3
"""Verify that release-facing package metadata matches a release tag."""

from __future__ import annotations

import argparse
import json
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
PACKAGE_MANIFESTS = {
    "smol-bytes": ROOT / "smol-bytes" / "Cargo.toml",
    "smol-bytes-py": ROOT / "smol-bytes-py" / "Cargo.toml",
    "smol-bytes-wasm": ROOT / "smol-bytes-wasm" / "Cargo.toml",
}


def cargo_version() -> str:
    with (ROOT / "Cargo.toml").open("rb") as source:
        workspace_manifest = tomllib.load(source)

    workspace_version = workspace_manifest["workspace"]["package"]["version"]
    if not isinstance(workspace_version, str) or not workspace_version:
        raise ValueError("workspace.package.version must be a non-empty string")

    for package_name, manifest_path in PACKAGE_MANIFESTS.items():
        with manifest_path.open("rb") as source:
            package = tomllib.load(source)["package"]

        if package.get("name") != package_name:
            raise ValueError(
                f"{manifest_path.relative_to(ROOT)} has package name "
                f"{package.get('name')!r}, expected {package_name!r}"
            )

        package_version = package["version"]
        if isinstance(package_version, dict) and package_version.get("workspace") is True:
            package_version = workspace_version
        if package_version != workspace_version:
            raise ValueError(
                f"{package_name} package version {package_version!r} does not match "
                f"workspace version {workspace_version!r}"
            )

    return workspace_version


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", help="expected release tag, including the leading v")
    parser.add_argument(
        "--check-js",
        action="store_true",
        help="also require js/package.json to use the Cargo version",
    )
    args = parser.parse_args()

    try:
        version = cargo_version()

        if args.tag is not None:
            if not args.tag.startswith("v") or len(args.tag) == 1:
                raise ValueError(f"release tag {args.tag!r} must have the form v<version>")
            if args.tag[1:] != version:
                raise ValueError(
                    f"release tag {args.tag!r} does not match Cargo version {version!r}"
                )

        if args.check_js:
            package_json = json.loads((ROOT / "js" / "package.json").read_text())
            js_version = package_json.get("version")
            if js_version != version:
                raise ValueError(
                    f"js/package.json version {js_version!r} does not match "
                    f"Cargo version {version!r}"
                )
    except (
        KeyError,
        OSError,
        TypeError,
        ValueError,
        json.JSONDecodeError,
        tomllib.TOMLDecodeError,
    ) as error:
        print(f"release version check failed: {error}", file=sys.stderr)
        return 1

    print(f"release version {version} is consistent")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
