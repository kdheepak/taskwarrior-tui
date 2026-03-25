#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = [
#   "mkdocs",
#   "mkdocs-exclude",
#   "mkdocs-macros-plugin",
#   "mkdocs-material",
#   "mkdocs-material-extensions",
#   "pymdown-extensions",
#   "pygments",
#   "termcolor",
# ]
# ///

from __future__ import annotations

import shutil
import subprocess
import sys
from pathlib import Path


def main() -> None:
    root = Path(__file__).resolve().parents[1]
    shutil.copyfile(root / "README.md", root / "docs" / "index.md")
    subprocess.run([sys.executable, "-m", "mkdocs", "build", *sys.argv[1:]], cwd=root, check=True)


if __name__ == "__main__":
    main()
