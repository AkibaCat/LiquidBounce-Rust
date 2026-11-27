#!/usr/bin/env python3
"""Validate shared placement kernel composition; serial owner applies."""
from pathlib import Path
import argparse,runpy
HERE=Path(__file__).resolve().parent
Plan=runpy.run_path(str(HERE/'integrate-super-knockback.py'))['Plan']
def build_plan():
    p=Plan()
    p.replace('native/liquidbounce-horizon/make-behavior-patch.py',"'step-patch.py'):","'block-placement-patch.py', 'step-patch.py'):",'placement kernel before final Step adapter scan')
    return p
def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--apply',action='store_true');args=parser.parse_args();p=build_plan()
    print(f'Block placement kernel: {len(p.steps)} edits/{len(p.text)} files validated')
    if args.apply:p.write();print('Block placement kernel integrated')
    else:print('Check only; no shared writes')
if __name__=='__main__':main()
