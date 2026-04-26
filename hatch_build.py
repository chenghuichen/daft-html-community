"""Hatchling build hook for daft-html.

During a real wheel build (`python -m build --wheel`):
  1. Locates the pre-compiled Rust cdylib in the Cargo output directory.
  2. Injects it into the wheel via force_include (no manual `cp` required).
  3. Stamps the wheel with the correct platform tag.

Environment variables (set by CI matrix):
  CARGO_BUILD_TARGET  - Rust target triple, e.g. "x86_64-unknown-linux-gnu"
                        Selects target/<CARGO_BUILD_TARGET>/release/ as the
                        artifact directory.  Omit for native builds where the
                        artifact lives in target/release/.
  WHEEL_PLAT_TAG      - Platform portion of the wheel tag, e.g.
                        "manylinux_2_17_x86_64.manylinux2014_x86_64" or
                        "macosx_11_0_arm64".  Omit to auto-detect from the
                        current OS/arch (useful for local `make build-whl`).

Editable installs (`pip install -e .`) skip all hook logic; the Makefile is
responsible for copying the .so into daft_html/ before the editable install.
"""

from __future__ import annotations

import os
import platform
from pathlib import Path

from hatchling.builders.hooks.plugin.interface import BuildHookInterface


class CustomBuildHook(BuildHookInterface):
    PLUGIN_NAME = "custom"

    def initialize(self, version: str, build_data: dict) -> None:
        if version == "editable":
            return

        target = os.environ.get("CARGO_BUILD_TARGET", "").strip()
        plat_tag = os.environ.get("WHEEL_PLAT_TAG", "").strip()

        # ── Locate the compiled artifact ──────────────────────────────────────
        if target:
            base = Path(f"target/{target}/release")
        else:
            base = Path("target/release")

        candidates = [
            *base.glob("libdaft_html*.so"),
            *base.glob("libdaft_html*.dylib"),
            *base.glob("daft_html*.dll"),
        ]
        if not candidates:
            raise FileNotFoundError(
                f"No compiled daft-html library found in {base}.\n"
                "Run `cargo build --release` (or `cargo zigbuild --release "
                "--target <triple>.<glibc>`) before calling `python -m build`."
            )
        artifact = candidates[0]

        # ── Inject artifact into wheel ────────────────────────────────────────
        # force_include maps the on-disk source path → destination path inside
        # the wheel archive.  This is equivalent to a manual cp + artifacts
        # entry but works without touching the source tree.
        dest_in_wheel = f"daft_html/{artifact.name}"
        build_data["force_include"][str(artifact)] = dest_in_wheel

        # ── Set platform-specific wheel tag ───────────────────────────────────
        if not plat_tag:
            plat_tag = _detect_plat_tag()

        if plat_tag:
            # "py3-none-{plat}" signals:
            #   py3   — any Python 3 (no CPython ABI dependency)
            #   none  — no stable ABI constraint
            #   {plat} — platform-specific native code
            build_data["tag"] = f"py3-none-{plat_tag}"


def _detect_plat_tag() -> str:
    """Auto-detect the platform tag for a local build (no CI env vars set)."""
    system = platform.system()
    machine = platform.machine()

    if system == "Linux":
        arch = "x86_64" if machine == "x86_64" else "aarch64"
        return f"manylinux_2_17_{arch}.manylinux2014_{arch}"
    elif system == "Darwin":
        arch = "arm64" if machine == "arm64" else "x86_64"
        min_ver = "11_0" if arch == "arm64" else "10_14"
        return f"macosx_{min_ver}_{arch}"
    elif system == "Windows":
        return "win_amd64"
    return ""
