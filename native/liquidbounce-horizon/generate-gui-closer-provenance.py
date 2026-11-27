#!/usr/bin/env python3
"""Hash the pinned crates.io archive, original unpacked files and local fork."""
from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parents[2]
def digest(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def main():
    cargo=Path.home()/'.cargo/registry'
    registry=next((cargo/'src').glob('*/java_regex-0.1.0'))
    archive=next((cargo/'cache').glob('*/java_regex-0.1.0.crate'))
    vendor=ROOT/'native/liquidbounce-behaviors/vendor/java_regex'
    files={}
    for path in sorted(vendor.rglob('*')):
        if not path.is_file() or path.name=='PROVENANCE.json':continue
        name=path.relative_to(vendor).as_posix();original=registry/name
        files[name]={'sha256':digest(path),'origin':'unchanged registry' if original.is_file() and original.read_bytes()==path.read_bytes() else 'locally changed registry' if original.is_file() else 'local addition'}
        if original.is_file():files[name]['registrySha256']=digest(original)
    report={'package':'java_regex','version':'0.1.0','registry':'crates.io','registryVcsRevision':json.loads((registry/'.cargo_vcs_info.json').read_text())['git']['sha1'],'crateArchiveSha256':digest(archive),'license':'MIT OR Apache-2.0; original notices retained. Local compatibility changes distributed with this GPL-3.0-or-later project.','compiledEntryPoint':'src/gui_api.rs','scope':'Narrow internal GUICloser Regex new/pattern/matches/contains_match API. No blanket equivalence claim from finite vectors.','jdkData':'jdk25 files are generated from observable JDK25 Character/Pattern behavior by JdkRegexFacts.java, not copied JDK implementation source.','jdkDataReport':{'path':'build/gui-closer-jdk-data-02/report.json','sha256':digest(ROOT/'build/gui-closer-jdk-data-02/report.json')},'differentialReport':{'path':'build/gui-closer-source-16/report.json','sha256':digest(ROOT/'build/gui-closer-source-16/report.json'),'vectors':45371},'files':files}
    (vendor/'PROVENANCE.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')
    print(json.dumps({'files':len(files),'manifestSha256':digest(vendor/'PROVENANCE.json')}))
if __name__=='__main__':main()
