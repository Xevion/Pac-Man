#!/usr/bin/env python3
"""
Pre-commit hook script to automatically bump Cargo.toml version based on commit message.

This script parses the commit message for version bump keywords and uses cargo set-version
to update the version in Cargo.toml accordingly.

Supported keywords:
- "major" or "breaking": Bump major version (1.0.0 -> 2.0.0)
- "minor" or "feature": Bump minor version (1.0.0 -> 1.1.0)
- "patch" or "fix" or "bugfix": Bump patch version (1.0.0 -> 1.0.1)

Usage: python scripts/bump-version.py <commit_message_file>
"""

import sys
import re
import subprocess
import os
from pathlib import Path


def get_current_version():
    """Get the current version from Cargo.toml."""
    try:
        result = subprocess.run(
            ["cargo", "metadata", "--format-version", "1", "--no-deps"],
            capture_output=True,
            text=True,
            check=True
        )

        # Parse the JSON output to get version
        import json
        metadata = json.loads(result.stdout)
        return metadata["packages"][0]["version"]
    except (subprocess.CalledProcessError, json.JSONDecodeError, KeyError) as e:
        print(f"Error getting current version: {e}", file=sys.stderr)
        return None


def bump_version(current_version, bump_type):
    """Calculate the new version based on bump type."""
    try:
        major, minor, patch = map(int, current_version.split('.'))

        if bump_type == "major":
            return f"{major + 1}.0.0"
        elif bump_type == "minor":
            return f"{major}.{minor + 1}.0"
        elif bump_type == "patch":
            return f"{major}.{minor}.{patch + 1}"
        else:
            return None
    except ValueError:
        print(f"Invalid version format: {current_version}", file=sys.stderr)
        return None


def set_version(new_version):
    """Set the new version using cargo set-version."""
    try:
        result = subprocess.run(
            ["cargo", "set-version", new_version],
            capture_output=True,
            text=True,
            check=True
        )
        print(f"Successfully bumped version to {new_version}")
        return True
    except subprocess.CalledProcessError as e:
        print(f"Error setting version: {e}", file=sys.stderr)
        print(f"stdout: {e.stdout}", file=sys.stderr)
        print(f"stderr: {e.stderr}", file=sys.stderr)
        return False


def parse_commit_message(commit_message_file):
    """Parse the commit message file for version bump keywords."""
    try:
        with open(commit_message_file, 'r', encoding='utf-8') as f:
            message = f.read().lower()
    except FileNotFoundError:
        print(f"Commit message file not found: {commit_message_file}", file=sys.stderr)
        return None
    except Exception as e:
        print(f"Error reading commit message: {e}", file=sys.stderr)
        return None

    # Check for version bump keywords
    if re.search(r'\b(major|breaking)\b', message):
        return "major"
    elif re.search(r'\b(minor|feature)\b', message):
        return "minor"
    elif re.search(r'\b(patch|fix|bugfix)\b', message):
        return "patch"

    return None


def main():
    if len(sys.argv) != 2:
        print("Usage: python scripts/bump-version.py <commit_message_file>", file=sys.stderr)
        sys.exit(1)

    commit_message_file = sys.argv[1]

    # Parse commit message for version bump type
    bump_type = parse_commit_message(commit_message_file)

    if not bump_type:
        print("No version bump keywords found in commit message")
        sys.exit(0)

    print(f"Found version bump type: {bump_type}")

    # Get current version
    current_version = get_current_version()
    if not current_version:
        print("Failed to get current version", file=sys.stderr)
        sys.exit(1)

    print(f"Current version: {current_version}")

    # Calculate new version
    new_version = bump_version(current_version, bump_type)
    if not new_version:
        print("Failed to calculate new version", file=sys.stderr)
        sys.exit(1)

    print(f"New version: {new_version}")

    # Set the new version
    if set_version(new_version):
        print("Version bump completed successfully")
        sys.exit(0)
    else:
        print("Version bump failed", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
