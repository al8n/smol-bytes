"""Focused stdlib tests for scripts/check-release-version.py."""

from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path


SCRIPT_PATH = Path(__file__).with_name("check-release-version.py")
SPEC = importlib.util.spec_from_file_location("check_release_version", SCRIPT_PATH)
assert SPEC is not None and SPEC.loader is not None
CHECK = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(CHECK)

VERSION = "0.2.0"


class ReleaseVersionCheckTests(unittest.TestCase):
    def make_root(self) -> Path:
        temporary = tempfile.TemporaryDirectory()
        self.addCleanup(temporary.cleanup)
        root = Path(temporary.name)
        (root / "Cargo.toml").write_text(
            f"[workspace.package]\nversion = \"{VERSION}\"\n",
        )
        for package_name in CHECK.PACKAGE_NAMES:
            package_dir = root / package_name
            package_dir.mkdir()
            (package_dir / "Cargo.toml").write_text(
                f"[package]\nname = \"{package_name}\"\nversion.workspace = true\n",
            )
        self.write_json(root, "js/package.json", {"name": "smol-bytes", "version": VERSION})
        self.write_json(
            root,
            "js/package-lock.json",
            {
                "name": "smol-bytes",
                "version": VERSION,
                "packages": {"": {"name": "smol-bytes", "version": VERSION}},
            },
        )
        return root

    @staticmethod
    def write_json(root: Path, relative: str, value: object) -> None:
        path = root / relative
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(json.dumps(value))

    def test_matching_metadata_succeeds(self) -> None:
        root = self.make_root()
        self.assertEqual(CHECK.verify_release(root, tag="v0.2.0", check_js=True), VERSION)

    def test_malformed_and_mismatched_tags_fail_independently(self) -> None:
        root = self.make_root()
        for tag in ("0.2.0", "v", "v0.1.8"):
            with self.subTest(tag=tag), self.assertRaises(ValueError):
                CHECK.verify_release(root, tag=tag)

    def test_cargo_member_name_mismatch_fails(self) -> None:
        root = self.make_root()
        (root / "smol-bytes" / "Cargo.toml").write_text(
            "[package]\nname = \"wrong\"\nversion.workspace = true\n",
        )
        with self.assertRaisesRegex(ValueError, r"expected 'smol-bytes'"):
            CHECK.verify_release(root)

    def test_cargo_member_explicit_version_mismatch_fails(self) -> None:
        root = self.make_root()
        (root / "smol-bytes-py" / "Cargo.toml").write_text(
            "[package]\nname = \"smol-bytes-py\"\nversion = \"0.1.8\"\n",
        )
        with self.assertRaisesRegex(ValueError, r"does not match workspace version"):
            CHECK.verify_release(root)

    def test_malformed_package_json_fails(self) -> None:
        root = self.make_root()
        (root / "js" / "package.json").write_text("{")
        with self.assertRaises(json.JSONDecodeError):
            CHECK.verify_release(root, check_js=True)

    def test_non_object_package_json_fails(self) -> None:
        root = self.make_root()
        self.write_json(root, "js/package.json", [])
        with self.assertRaisesRegex(ValueError, r"package.json must contain a JSON object"):
            CHECK.verify_release(root, check_js=True)

    def test_package_json_identity_mismatches_fail(self) -> None:
        for metadata, pattern in (
            ({"name": "wrong", "version": VERSION}, r"package.json root package name"),
            ({"name": "smol-bytes", "version": "0.1.8"}, r"package.json root package version"),
        ):
            with self.subTest(metadata=metadata):
                root = self.make_root()
                self.write_json(root, "js/package.json", metadata)
                with self.assertRaisesRegex(ValueError, pattern):
                    CHECK.verify_release(root, check_js=True)

    def test_malformed_package_lock_fails(self) -> None:
        root = self.make_root()
        (root / "js" / "package-lock.json").write_text("{")
        with self.assertRaises(json.JSONDecodeError):
            CHECK.verify_release(root, check_js=True)

    def test_non_object_package_lock_fails(self) -> None:
        root = self.make_root()
        self.write_json(root, "js/package-lock.json", [])
        with self.assertRaisesRegex(ValueError, r"package-lock.json must contain a JSON object"):
            CHECK.verify_release(root, check_js=True)

    def test_package_lock_top_level_identity_mismatches_fail(self) -> None:
        for lock, pattern in (
            (
                {"name": "wrong", "version": VERSION, "packages": {"": {"name": "smol-bytes", "version": VERSION}}},
                r"top-level package name",
            ),
            (
                {"name": "smol-bytes", "version": "0.1.8", "packages": {"": {"name": "smol-bytes", "version": VERSION}}},
                r"top-level package version",
            ),
        ):
            with self.subTest(lock=lock):
                root = self.make_root()
                self.write_json(root, "js/package-lock.json", lock)
                with self.assertRaisesRegex(ValueError, pattern):
                    CHECK.verify_release(root, check_js=True)

    def test_missing_or_non_object_packages_fails(self) -> None:
        for packages, pattern in ((None, r"packages must be"), ([], r"packages must be a JSON object")):
            with self.subTest(packages=packages):
                root = self.make_root()
                lock = {"name": "smol-bytes", "version": VERSION}
                if packages is not None:
                    lock["packages"] = packages
                self.write_json(root, "js/package-lock.json", lock)
                with self.assertRaisesRegex(ValueError, pattern):
                    CHECK.verify_release(root, check_js=True)

    def test_missing_or_non_object_root_package_fails(self) -> None:
        for root_package, pattern in ((None, r'packages\[""\] must be'), ([], r'packages\[""\] must be a JSON object')):
            with self.subTest(root_package=root_package):
                root = self.make_root()
                packages: dict[str, object] = {}
                if root_package is not None:
                    packages[""] = root_package
                self.write_json(
                    root,
                    "js/package-lock.json",
                    {"name": "smol-bytes", "version": VERSION, "packages": packages},
                )
                with self.assertRaisesRegex(ValueError, pattern):
                    CHECK.verify_release(root, check_js=True)

    def test_package_lock_root_entry_identity_mismatches_fail(self) -> None:
        for root_entry, pattern in (
            ({"name": "wrong", "version": VERSION}, r'packages\[""\] package name'),
            ({"name": "smol-bytes", "version": "0.1.8"}, r'packages\[""\] package version'),
        ):
            with self.subTest(root_entry=root_entry):
                root = self.make_root()
                self.write_json(
                    root,
                    "js/package-lock.json",
                    {"name": "smol-bytes", "version": VERSION, "packages": {"": root_entry}},
                )
                with self.assertRaisesRegex(ValueError, pattern):
                    CHECK.verify_release(root, check_js=True)


if __name__ == "__main__":
    unittest.main()
