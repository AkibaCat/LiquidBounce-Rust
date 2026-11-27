#!/usr/bin/env python3
"""Check the actual staging reader and pinned legacy source bundle layout."""
from pathlib import Path
import argparse,hashlib,importlib.util,json,zipfile
ROOT=Path(__file__).resolve().parents[2]

def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args()
    out=(ROOT/args.out).resolve();out.mkdir(parents=True,exist_ok=True);report=out/'report.json'
    if report.exists():raise SystemExit('Refusing to overwrite evidence')
    spec=importlib.util.spec_from_file_location('packaging',ROOT/'native/liquidbounce-horizon/package.py')
    packaging=importlib.util.module_from_spec(spec);spec.loader.exec_module(packaging)
    source=ROOT/'native/liquidbounce-behaviors';staging=packaging.read_package_source(source)
    vendor=[name for name in staging if name.startswith('vendor/java_regex/')]
    assert len(vendor)==24,(len(vendor),vendor)
    files=packaging.package_files(packaging.verify_baseline());candidate=out/'gui-closer-source.hnmod'
    with zipfile.ZipFile(candidate,'w',zipfile.ZIP_DEFLATED) as archive:
        for name,data in sorted(files.items()):archive.writestr(name,data)
    validation=packaging.validate_package(candidate)
    for name in vendor:
        relative='data/java_regex/'+name.removeprefix('vendor/java_regex/')
        assert files[packaging.PREFIX+'behaviors/'+relative]==(source/name).read_bytes(),name
    assert b'path = "data/java_regex"' in files[packaging.PREFIX+'behaviors/Cargo.toml']
    result={'passed':True,'scope':'Actual read_package_source staging inputs include all24 vendor files; actual legacy source bundle preserves every vendor file byte-for-byte under behaviors/data/java_regex and rewrites only behavior manifest dependency path. Complete production package validation passes. Does not claim patched-game loading from the legacy unpatched trait bundle.','vendorFiles':{name:hashlib.sha256(staging[name]).hexdigest() for name in vendor},'sourcePackageValidation':validation,'productionInputs':{str(path.relative_to(ROOT)).replace('\\','/'):hashlib.sha256(path.read_bytes()).hexdigest() for path in [ROOT/'native/liquidbounce-horizon/package.py',ROOT/'scripts/stage-horizon-build.py',source/'Cargo.toml']}}
    report.write_text(json.dumps(result,indent=2)+'\n',encoding='utf8')
    print(json.dumps({'passed':True,'report':str(report),'vendorFiles':len(vendor)}))

if __name__=='__main__':main()
