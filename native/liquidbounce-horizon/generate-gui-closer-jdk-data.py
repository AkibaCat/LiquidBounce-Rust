#!/usr/bin/env python3
"""Generate pinned observable JDK character facts, without copying JDK source."""
from pathlib import Path
import argparse,hashlib,json,subprocess,time
ROOT=Path(__file__).resolve().parents[2]
def main():
    parser=argparse.ArgumentParser();parser.add_argument('--out',required=True);args=parser.parse_args()
    out=(ROOT/args.out).resolve();out.mkdir(parents=True,exist_ok=True)
    if (out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
    source=ROOT/'native/liquidbounce-horizon/gui-closer-source-fixture/JdkRegexFacts.java'
    copied=out/source.name;copied.write_bytes(source.read_bytes())
    target=ROOT/'native/liquidbounce-behaviors/vendor/java_regex/jdk25'
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe'
    argv=[str(java),'--add-opens=java.base/java.util.regex=ALL-UNNAMED','--add-opens=java.base/java.lang=ALL-UNNAMED','--add-opens=java.base/jdk.internal.util.regex=ALL-UNNAMED',str(copied),str(target)]
    start=time.monotonic();result=subprocess.run(argv,cwd=ROOT,capture_output=True,text=True,encoding='utf8',errors='replace')
    (out/'stdout.log').write_text(result.stdout,encoding='utf8');(out/'stderr.log').write_text(result.stderr,encoding='utf8')
    report={'passed':result.returncode==0,'scope':'Observable Character/CharPredicates/Grapheme facts from actual JDK25, no JDK source copied.','argv':argv,'exitCode':result.returncode,'seconds':round(time.monotonic()-start,3),'generatorSha256':hashlib.sha256(source.read_bytes()).hexdigest(),'data':{p.name:{'bytes':p.stat().st_size,'sha256':hashlib.sha256(p.read_bytes()).hexdigest()}for p in target.glob('*')if p.is_file()}}
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps(report))
if __name__=='__main__':main()
