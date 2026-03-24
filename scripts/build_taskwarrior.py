#!/usr/bin/env -S uv run --script
# /// script
# requires-python = ">=3.11"
# dependencies = []
# ///

from __future__ import annotations

import argparse
import os
import shlex
import shutil
import subprocess
import sys
from pathlib import Path


DEFAULT_REPO_URL = "https://github.com/GothenburgBitFactory/taskwarrior.git"


def fail(message: str) -> None:
    print(message, file=sys.stderr)
    raise SystemExit(1)


def ensure_command(name: str) -> None:
    if shutil.which(name) is None:
        fail(f"Required command not found on PATH: {name}")


def format_command(args: list[str]) -> str:
    return " ".join(shlex.quote(arg) for arg in args)


def run(args: list[str], *, cwd: Path | None = None) -> None:
    print(f"+ {format_command(args)}")
    subprocess.run(args, cwd=cwd, check=True)


def capture(args: list[str], *, cwd: Path | None = None) -> str:
    completed = subprocess.run(
        args,
        cwd=cwd,
        check=True,
        text=True,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    return completed.stdout.strip()


def env_or(value: str | None, env_var: str, default: str) -> str:
    return value if value is not None else os.getenv(env_var, default)


def resolve_path(value: str) -> Path:
    return Path(value).expanduser()


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Clone/update upstream Taskwarrior and build the task binary from source."
    )
    parser.add_argument("--repo-url")
    parser.add_argument("--branch")
    parser.add_argument("--source-dir")
    parser.add_argument("--build-dir")
    parser.add_argument("--install-dir")
    parser.add_argument("--build-type")
    return parser.parse_args()


def main() -> None:
    args = parse_args()

    repo_url = env_or(args.repo_url, "TASKWARRIOR_REPO_URL", DEFAULT_REPO_URL)
    branch = env_or(args.branch, "TASKWARRIOR_BRANCH", "stable")
    source_dir = resolve_path(env_or(args.source_dir, "TASKWARRIOR_SOURCE_DIR", "target/taskwarrior-src"))
    build_dir = resolve_path(
        env_or(args.build_dir, "TASKWARRIOR_BUILD_DIR", f"target/taskwarrior-build/{branch}")
    )
    install_dir = resolve_path(
        env_or(args.install_dir, "TASKWARRIOR_INSTALL_DIR", f"target/taskwarrior-install/{branch}")
    )

    if args.build_type is not None:
        build_type = args.build_type
    else:
        build_type = os.getenv("TASKWARRIOR_BUILD_TYPE", "Debug" if branch == "develop" else "Release")

    ensure_command("git")
    ensure_command("cmake")

    source_dir.parent.mkdir(parents=True, exist_ok=True)

    if not source_dir.exists():
        run(["git", "clone", "--recurse-submodules", repo_url, os.fspath(source_dir)])
    elif not (source_dir / ".git").is_dir():
        fail(f"TASKWARRIOR_SOURCE_DIR={source_dir} exists but is not a git checkout.")

    if capture(["git", "status", "--porcelain"], cwd=source_dir):
        fail(f"Refusing to update {source_dir} because it has local changes.")

    run(["git", "fetch", "--prune", "origin"], cwd=source_dir)

    has_local_branch = subprocess.run(
        ["git", "show-ref", "--verify", "--quiet", f"refs/heads/{branch}"],
        cwd=source_dir,
        check=False,
    ).returncode == 0

    if has_local_branch:
        run(["git", "switch", branch], cwd=source_dir)
    else:
        run(["git", "switch", "-c", branch, "--track", f"origin/{branch}"], cwd=source_dir)

    run(["git", "pull", "--ff-only", "origin", branch], cwd=source_dir)
    run(["git", "submodule", "sync", "--recursive"], cwd=source_dir)
    run(["git", "submodule", "update", "--init", "--recursive"], cwd=source_dir)

    run(
        [
            "cmake",
            "-S",
            os.fspath(source_dir),
            "-B",
            os.fspath(build_dir),
            f"-DCMAKE_BUILD_TYPE={build_type}",
            f"-DCMAKE_INSTALL_PREFIX={install_dir}",
        ]
    )
    run(["cmake", "--build", os.fspath(build_dir), "--parallel"])
    run(["cmake", "--install", os.fspath(build_dir)])

    task_bin = install_dir / "bin" / ("task.exe" if os.name == "nt" else "task")
    if not task_bin.is_file():
        fail(f"Expected built task binary at {task_bin}")

    commit = capture(["git", "rev-parse", "--short", "HEAD"], cwd=source_dir)
    print(f"Built Taskwarrior from {branch} ({commit}): {task_bin}")
    run([os.fspath(task_bin), "--version"])


if __name__ == "__main__":
    main()
