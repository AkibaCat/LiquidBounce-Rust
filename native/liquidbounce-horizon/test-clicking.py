#!/usr/bin/env python3
"""One focused smoke for the source-derived shared Clicker dependency, no Cargo."""
from pathlib import Path
import subprocess,tempfile
HERE=Path(__file__).resolve().parent
def main():
    with tempfile.TemporaryDirectory(prefix='liquidbounce-clicking-') as temp:
        exe=Path(temp)/'clicking-smoke.exe'
        subprocess.run(['rustc','--edition=2024','--test',str(HERE/'clicking-smoke.rs'),'-o',str(exe)],check=True)
        subprocess.run([str(exe),'--nocapture'],check=True)
if __name__=='__main__':main()
