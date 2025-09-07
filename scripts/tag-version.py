#!/usr/bin/env python3
"""
Post-commit hook script to automatically create git tags based on the version in Cargo.toml.

This script reads the current version from Cargo.toml and creates a git tag with that version.
It's designed to run after the version has been bumped by the bump-version.py script.

Usage: python scripts/tag-version.py
"""

import sys
import subprocess
import re
from pathlib import Path


def get_version_from_cargo_toml():
    """Get the current version from Cargo.toml."""
    cargo_toml_path = Path("Cargo.toml")

    if not cargo_toml_path.exists():
        print("Cargo.toml not found", file=sys.stderr)
        return None

    try:
        with open(cargo_toml_path, 'r', encoding='utf-8') as f:
            content = f.read()

        # Look for version = "x.y.z" pattern
        version_match = re.search(r'version\s*=\s*["\']([^"\']+)["\']', content)

        if version_match:
            return version_match.group(1)
        else:
            print("Could not find version in Cargo.toml", file=sys.stderr)
            return None

    except Exception as e:
        print(f"Error reading Cargo.toml: {e}", file=sys.stderr)
        return None


def get_existing_tags():
    """Get list of existing git tags."""
    try:
        result = subprocess.run(
            ["git", "tag", "--list"],
            capture_output=True,
            text=True,
            check=True
        )
        return result.stdout.strip().split('\n') if result.stdout.strip() else []
    except subprocess.CalledProcessError as e:
        print(f"Error getting git tags: {e}", file=sys.stderr)
        return []


def create_git_tag(version):
    """Create a git tag with the specified version."""
    tag_name = f"v{version}"

    try:
        # Check if tag already exists
        existing_tags = get_existing_tags()
        if tag_name in existing_tags:
            print(f"Tag {tag_name} already exists, skipping")
            return True

        # Create the tag
        result = subprocess.run(
            ["git", "tag", tag_name],
            capture_output=True,
            text=True,
            check=True
        )

        print(f"Successfully created tag: {tag_name}")
        return True

    except subprocess.CalledProcessError as e:
        print(f"Error creating git tag: {e}", file=sys.stderr)
        print(f"stdout: {e.stdout}", file=sys.stderr)
        print(f"stderr: {e.stderr}", file=sys.stderr)
        return False


def is_git_repository():
    """Check if we're in a git repository."""
    try:
        subprocess.run(
            ["git", "rev-parse", "--git-dir"],
            capture_output=True,
            check=True
        )
        return True
    except subprocess.CalledProcessError:
        return False


def main():
    # Check if we're in a git repository
    if not is_git_repository():
        print("Not in a git repository, skipping tag creation")
        sys.exit(0)

    # Get the current version from Cargo.toml
    version = get_version_from_cargo_toml()

    if not version:
        print("Could not determine version, skipping tag creation")
        sys.exit(0)

    print(f"Current version: {version}")

    # Create the git tag
    if create_git_tag(version):
        print("Tag creation completed successfully")
        sys.exit(0)
    else:
        print("Tag creation failed", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
