#!/usr/bin/env -S uv run
# /// script
# requires-python = ">=3.11"
# ///

from __future__ import annotations

import os
import shlex
import subprocess
import sys


def split_csv(value: str) -> list[str]:
    return [item.strip() for item in value.split(",") if item.strip()]


def run(cmd: list[str]) -> None:
    print("+", " ".join(shlex.quote(part) for part in cmd), flush=True)
    subprocess.run(cmd, check=True)


def main() -> int:
    toolchain = os.environ.get("RUSTUP_TOOLCHAIN")
    if not toolchain:
        print("RUSTUP_TOOLCHAIN is not set", file=sys.stderr)
        return 1

    profile = os.environ.get("RUST_TOOLCHAIN_PROFILE", "default")
    components = split_csv(os.environ.get("RUST_TOOLCHAIN_COMPONENTS", ""))
    targets = split_csv(os.environ.get("RUST_TOOLCHAIN_TARGETS", ""))

    run(["rustup", "toolchain", "install", toolchain, "--profile", profile])

    if components:
        run(["rustup", "component", "add", f"--toolchain={toolchain}", *components])

    if targets:
        run(["rustup", "target", "add", f"--toolchain={toolchain}", *targets])

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
