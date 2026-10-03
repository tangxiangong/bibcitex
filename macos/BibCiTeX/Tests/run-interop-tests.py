#!/usr/bin/env python3
"""Link/run generated Swift/Rust ABI tests, without launching the application."""
import argparse
from pathlib import Path
import platform
import subprocess
import tempfile

package = Path(__file__).resolve().parent.parent
repo = package.parent.parent
architecture = platform.machine()
arguments = argparse.ArgumentParser()
arguments.add_argument("--products", type=Path, default=repo / "target/swift" / architecture / "apple/Products/Debug")
arguments.add_argument("--rust-library", type=Path, default=repo / "target/debug/libbibcitex_ffi.a")
options = arguments.parse_args()
for required in [options.products / "BibCiTeXCore_Module.o", options.rust_library]:
    if not required.exists():
        raise SystemExit(f"Build the Swift app and Rust library first: missing {required}")
with tempfile.TemporaryDirectory(prefix="bibcitex-interop-tests-") as directory:
    work = Path(directory)
    executable = work / "interop-tests"
    command = [
        "swiftc", "-parse-as-library", "-target", f"{architecture}-apple-macosx13.0",
        "-module-cache-path", str(work / "modules"),
        "-I", str(options.products), "-I", str(package / "Generated/BibCiTeXCoreFFI"),
        str(package / "Tests/RustInteropTests.swift"),
        str(options.products / "BibCiTeXCore_Module.o"), str(options.rust_library),
        "-framework", "AppKit", "-framework", "Carbon", "-framework", "ApplicationServices",
        "-framework", "CoreGraphics", "-framework", "Foundation", "-liconv",
        "-o", str(executable),
    ]
    subprocess.run(command, check=True)
    subprocess.run([str(executable)], check=True, timeout=30)
