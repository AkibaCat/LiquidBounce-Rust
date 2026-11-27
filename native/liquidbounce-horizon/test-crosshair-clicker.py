#!/usr/bin/env python3
"""One focused TriggerBot/AutoClicker behavior smoke, without Cargo or a client."""
from pathlib import Path
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent


def main():
    with tempfile.TemporaryDirectory(prefix="liquidbounce-crosshair-") as temp:
        exe = Path(temp) / "crosshair-clicker-smoke.exe"
        subprocess.run(
            ["rustc", "--edition=2024", "--test", str(HERE / "crosshair-clicker-smoke.rs"), "-o", str(exe)],
            check=True,
        )
        subprocess.run([str(exe), "--nocapture"], check=True)


if __name__ == "__main__":
    main()
