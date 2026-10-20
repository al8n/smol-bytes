#!/usr/bin/env python3
"""Verify that release-facing package metadata matches a release tag."""

from __future__ import annotations

import argparse
import json
import sys
import tomllib
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
PACKAGE_NAMES = ("smol-bytes", "smol-bytes-py", "smol-bytes-wasm")
JS_PACKAGE_NAME = "smol-bytes"


def cargo_version(root: Path = ROOT) -> str:
    """Return the workspace version after checking every inheriting package."""
    with (root / "Cargo.toml").open("rb") as source:
        workspace_manifest = tomllib.load(source)

    workspace_version = workspace_manifest["workspace"]["package"]["version"]
    if not isinstance(workspace_version, str) or not workspace_version:
        raise ValueError("workspace.package.version must be a non-empty string")

    for package_name in PACKAGE_NAMES:
        manifest_path = root / package_name / "Cargo.toml"
        with manifest_path.open("rb") as source:
            package = tomllib.load(source)["package"]

        if package.get("name") != package_name:
            raise ValueError(
                f"{manifest_path.relative_to(root)} has package name "
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


def read_json(path: Path) -> dict[str, Any]:
    """Read a JSON object with a useful error for malformed metadata."""
    value = json.loads(path.read_text())
    if not isinstance(value, dict):
        raise ValueError(f"{path.name} must contain a JSON object")
    return value


def require_js_identity(
    metadata: dict[str, Any],
    path: Path,
    location: str,
    version: str,
) -> None:
    """Check one npm metadata object against the canonical package identity."""
    name = metadata.get("name")
    if name != JS_PACKAGE_NAME:
        raise ValueError(
            f"{path} {location} package name {name!r} does not match "
            f"expected {JS_PACKAGE_NAME!r}"
        )

    js_version = metadata.get("version")
    if js_version != version:
        raise ValueError(
            f"{path} {location} package version {js_version!r} does not match "
            f"Cargo version {version!r}"
        )


def check_js_metadata(root: Path, version: str) -> None:
    """Require both npm manifests to agree with Cargo's package identity."""
    package_json_path = root / "js" / "package.json"
    package_lock_path = root / "js" / "package-lock.json"

    require_js_identity(read_json(package_json_path), package_json_path.relative_to(root), "root", version)

    package_lock = read_json(package_lock_path)
    relative_lock = package_lock_path.relative_to(root)
    require_js_identity(package_lock, relative_lock, "top-level", version)

    packages = package_lock.get("packages")
    if not isinstance(packages, dict):
        raise ValueError(f"{relative_lock} packages must be a JSON object")
    root_package = packages.get("")
    if not isinstance(root_package, dict):
        raise ValueError(f"{relative_lock} packages[\"\"] must be a JSON object")
    require_js_identity(root_package, relative_lock, 'packages[""]', version)


def verify_release(root: Path = ROOT, tag: str | None = None, check_js: bool = False) -> str:
    """Verify metadata at *root* and return the consistent Cargo version."""
    version = cargo_version(root)

    if tag is not None:
        if not tag.startswith("v") or len(tag) == 1:
            raise ValueError(f"release tag {tag!r} must have the form v<version>")
        if tag[1:] != version:
            raise ValueError(f"release tag {tag!r} does not match Cargo version {version!r}")

    if check_js:
        check_js_metadata(root, version)

    return version


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tag", help="expected release tag, including the leading v")
    parser.add_argument(
        "--check-js",
        action="store_true",
        help="also require npm package.json and package-lock.json metadata to match Cargo",
    )
    args = parser.parse_args()

    try:
        version = verify_release(tag=args.tag, check_js=args.check_js)
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
