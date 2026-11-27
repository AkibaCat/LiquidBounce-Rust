#!/usr/bin/env python3
"""Short untouched TargetLock and ListValue oracle with a constant clock provider."""
from pathlib import Path
import argparse,hashlib,json,os,runpy,shutil,struct,subprocess
HERE=Path(__file__).resolve().parent;ROOT=HERE.parents[1]
def bits(v,fmt='d'):return str(int.from_bytes(struct.pack('>'+fmt,v),'big'))
def player(i,name='Notch',distance=0,**extra):return dict(op='player',id=i,name=name,distance=bits(distance),**extra)
def op(name,value=None,**extra):return dict(op=name,**({}if value is None else {'value':value}),**extra)
def inputs():
 return [
 [player(1),player(2,'Other'),op('now','1000'),op('attack',id=1,cancelled=True),op('tag',id=2),op('attack',id=1),op('now','31000'),op('tick'),op('now','31001'),op('tick'),op('tag',id=2)],
 [player(1),op('notice',time=2,clock='8000'),op('attack',id=1),op('notice'),op('now','10001'),op('tick')],
 [player(1),player(2,'Other'),op('attack',id=2),op('attack',id=1),player(2,'Renamed',400),player(1,'Notch',0,removed=True),op('now','99999'),op('tick')],
 [op('tick'),player(1),op('attack',id=1),op('dead',True),op('tick'),op('dead',False),op('empty','AllowNone'),op('combat',True),op('tag',id=1)],
 [player(1),op('attack',id=1),op('running',False),op('now','99999'),op('tick'),op('running',True),op('mode','Filter'),op('tick'),op('mode','Temporary'),op('remove',id=1),op('tick')],
 [player(1,'İ'),player(2,'x'),op('mode','Filter'),op('names',[None,'i','i',False,42]),op('tag',id=1),op('tag',id=2),op('filter','Blacklist'),op('tag',id=1),op('tag',id=2),op('names',['ok',{}]),op('names',True),op('names',value=None)],
 [player(1),op('time',0),op('now','9223372036854775807'),op('attack',id=1),op('tick'),op('time',1),op('attack',id=1),op('tick')],
 [player(1,distance=1),op('attack',id=1),op('range',bits=bits(float('nan'),'f')),op('tick'),player(2,'Fake',client=False),op('attack',id=2),op('tag',id=2)],
 ]
def main():
 p=argparse.ArgumentParser();p.add_argument('--out',required=True);a=p.parse_args();out=ROOT/a.out
 if(out/'report.json').exists():raise SystemExit('Refusing to overwrite evidence')
 work=out/'original';classes=work/'classes';classes.mkdir(parents=True,exist_ok=True)
 base=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce'
 originals=[base/'features/module/modules/misc/ModuleTargetLock.kt',base/'config/types/list/ListValue.kt']
 for path in originals:shutil.copyfile(path,work/path.name)
 for path in(HERE/'target-lock-source-fixture').glob('*.kt'):shutil.copyfile(path,work/path.name)
 for name in ['fastutil.kt','input.kt']:shutil.copyfile(HERE/'gui-closer-config-source-fixture'/name,work/name)
 rows=inputs();rows[5][-1]['value']=None;(out/'inputs.json').write_text(json.dumps(rows),encoding='utf8')
 cache=Path(os.environ['USERPROFILE'])/'.gradle/caches/modules-2/files-2.1';gradle=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';jdk=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin'
 jars=[*gradle.glob('kotlin-stdlib-*.jar'),*(cache/'com.google.code.gson').rglob('*.jar'),*(cache/'it.unimi.dsi').rglob('*.jar')]
 cp=os.pathsep.join(map(str,[classes,*jars]));commands=[];clock=[]
 compile=[str(jdk/'java.exe'),'-cp',str(gradle/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-jvm-target','25','-no-stdlib','-no-reflect','-classpath',cp,'-d',str(classes),*map(str,work.glob('*.kt'))]
 r=subprocess.run(compile,cwd=ROOT,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'compile.log').write_text(r.stdout+r.stderr,encoding='utf8');commands.append({'name':'compile','exitCode':r.returncode})
 if r.returncode==0:
  provider=runpy.run_path(str(HERE/'collect-anti-hunger-original.py'))['clock_provider']
  for path in classes.rglob('ModuleTargetLock*.class'):
   original=path.read_bytes()
   if b'java/lang/System'in original:
    path.write_bytes(provider(original));clock.append({'path':str(path.relative_to(classes)),'originalSha256':hashlib.sha256(original).hexdigest(),'providerSha256':hashlib.sha256(path.read_bytes()).hexdigest()})
  assert len(clock)==1,clock
  r=subprocess.run([str(jdk/'java.exe'),'-cp',cp,'fixture.Fixture',str(out/'inputs.json'),str(out/'oracle.json')],cwd=ROOT,capture_output=True,text=True,encoding='utf8',errors='replace');(out/'run.log').write_text(r.stdout+r.stderr,encoding='utf8');commands.append({'name':'run','exitCode':r.returncode})
 report={'passed':len(commands)==2 and all(c['exitCode']==0 for c in commands),'scenarios':len(rows),'steps':sum(map(len,rows)),'commands':commands,'clockProvider':clock,'unchangedOriginalKotlin':all(p.read_bytes()==(work/p.name).read_bytes()for p in originals),'sources':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in originals},'scope':'Byte-identical original TargetLock and ListValue, actual fastutil linked map/Gson String adapter and JDK25; explicit mode/event/player/world/notification/settings providers. Only System class constant replaced. Short usability oracle, no complete original client or actual game proof.'}
 (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps({'passed':report['passed'],'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)
if __name__=='__main__':main()
