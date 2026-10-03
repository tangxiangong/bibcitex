#!/usr/bin/env python3
"""Compile the production model with an actor test double, without linking Rust."""
from pathlib import Path
import subprocess
import tempfile

package = Path(__file__).resolve().parent.parent
source = package / "Sources/BibCiTeX"
# The service calls generated Rust bindings, so only its theme declarations are needed here.
theme = (source / "HelperTheme.swift").read_text().split("actor HelperService:")[0]
with tempfile.TemporaryDirectory(prefix="bibcitex-regression-") as directory:
    work = Path(directory)
    combined = work / "Regression.swift"
    combined.write_text("\n".join([
        (source / "HelperModels.swift").read_text(),
        theme,
        (source / "HelperViewModel.swift").read_text(),
        (package / "Tests/HelperViewModelRegression.swift").read_text(),
    ]))
    executable = work / "regressions"
    subprocess.run(["swiftc", "-parse-as-library", "-module-cache-path", str(work / "modules"), str(combined), "-o", str(executable)], check=True)
    subprocess.run([str(executable)], check=True, timeout=20)
