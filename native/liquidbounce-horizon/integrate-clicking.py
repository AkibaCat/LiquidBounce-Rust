#!/usr/bin/env python3
"""Export the reusable Clicker dependency; integration owner uses --apply."""
from pathlib import Path
import argparse,importlib.util
HERE=Path(__file__).resolve().parent
spec=importlib.util.spec_from_file_location('clicking_shared',HERE/'integrate-super-knockback.py')
shared=importlib.util.module_from_spec(spec);spec.loader.exec_module(shared)
def build_plan():
    p=shared.Plan()
    p.replace('native/liquidbounce-behaviors/src/lib.rs','pub mod clock;\n','pub mod clock;\npub mod clicking;\n','shared Clicker export')
    return p
def main():
    parser=argparse.ArgumentParser(description=__doc__);mode=parser.add_mutually_exclusive_group()
    mode.add_argument('--check',action='store_true');mode.add_argument('--apply',action='store_true');args=parser.parse_args()
    plan=build_plan();print(f'Validated {len(plan.steps)} Clicker dependency changes across {len(plan.text)} files')
    if args.apply:plan.write();print('Applied Clicker dependency export')
    else:print('Check only; no shared file writes')
if __name__=='__main__':main()
