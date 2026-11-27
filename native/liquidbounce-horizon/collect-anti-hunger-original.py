#!/usr/bin/env python3
"""Untouched AntiHunger/Chronometer with explicit packet/storage/clock providers."""
import hashlib,itertools,json,os,shutil,struct,subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
OUT=ROOT/'build/anti-hunger-tests'
def d(value):return str(struct.unpack('>Q',struct.pack('>d',value))[0])
def context(flags=0,fall=0.):return dict(op='context',flags=flags,fall=d(fall))
def settings(no=True,swim=False,floating=True):return dict(op='settings',noSprint=no,whileSwimming=swim,floating=floating)
def packet(kind,now=1000,**kw):return dict(op='packet',kind=kind,clock=[str(now),str(now+1)],**kw)
def inputs():
 rows=[]
 for flags,fall,floating,running,ground in itertools.product(range(32),[-0.,0.,-1.,1.,float('nan'),float('inf')],[False,True],[False,True],[False,True]):
  rows.append([settings(floating=floating),dict(op='running',value=running),context(flags,fall),packet('move',ground=ground,cancelled=True)])
 for no,swim,active,running,source,value in itertools.product([False,True],[False,True],[False,True],[False,True],['INPUT','NETWORK','MOVEMENT'],[False,True]):
  rows.append([settings(no,swim),dict(op='running',value=running),context(int(active)),dict(op='sprint',source=source,value=value)])
 for no,swim,active,running,kind,now,cancelled in itertools.product([False,True],[False,True],[False,True],[False,True],['startSprint','stopSprint','startBreak','stopBreak','other'],[499,500,501,1000],[False,True]):
  rows.append([settings(no,swim),dict(op='running',value=running),context(int(active)),packet(kind,now,cancelled=cancelled)])
 for first,second in itertools.product(['startSprint','startBreak'],repeat=2):
  rows.append([packet(first,1000),packet(second,1501),packet(second,1502),dict(op='running',value=False),packet(first,5000),dict(op='running',value=True),packet(first,2003)])
 for last,now in [(2**63-200,1000),(-2**63,-2**63+500),(1234,1734),(1234,1735)]:rows.append([dict(op='last',value=str(last)),packet('startSprint',now)])
 return rows
def clock_provider(data):
 """Replace only Chronometer's System class constant; all method code is intact."""
 count=int.from_bytes(data[8:10],'big');pos=10;parts=[data[:10]];i=1;replaced=0
 while i<count:
  start=pos;tag=data[pos];pos+=1
  if tag==1:
   size=int.from_bytes(data[pos:pos+2],'big');pos+=2;value=data[pos:pos+size];pos+=size
   if value==b'java/lang/System':value=b'fixture/Clock';replaced+=1
   parts.append(bytes([tag])+len(value).to_bytes(2,'big')+value)
  else:
   length={3:4,4:4,5:8,6:8,7:2,8:2,9:4,10:4,11:4,12:4,15:3,16:2,17:4,18:4,19:2,20:2}[tag];pos+=length;parts.append(data[start:pos])
   if tag in [5,6]:i+=1
  i+=1
 if replaced!=1:raise ValueError(f'Expected one System class constant, got {replaced}')
 return b''.join(parts)+data[pos:]
def main():
 work=OUT/'original';classes=work/'classes';classes.mkdir(parents=True,exist_ok=True)
 base=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce'
 originals=[base/'features/module/modules/exploit/ModuleAntiHunger.kt',base/'utils/client/Chronometer.kt']
 for p in originals:shutil.copyfile(p,work/p.name)
 for p in (HERE/'oracles/anti-hunger').glob('*.kt'):shutil.copyfile(p,work/p.name)
 vectors=inputs();(OUT/'inputs.json').write_text(json.dumps(vectors),encoding='utf-8')
 cache=Path(os.environ['USERPROFILE'])/'.gradle/caches';gradle=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';jdk=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin'
 jars=[*gradle.glob('kotlin-stdlib-*.jar'),*(cache/'modules-2/files-2.1/com.google.code.gson').rglob('*.jar')]
 cp=os.pathsep.join(map(str,[classes,*jars]));commands=[]
 compile=[str(jdk/'java.exe'),'-cp',str(gradle/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',cp,'-d',str(classes),*map(str,work.glob('*.kt'))]
 r=subprocess.run(compile,cwd=ROOT,capture_output=True,text=True,encoding='utf-8',errors='replace');(OUT/'compile.log').write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr,end='');commands.append(dict(name='compile',exitCode=r.returncode))
 if r.returncode:raise SystemExit(r.returncode)
 chronometer=classes/'net/ccbluex/liquidbounce/utils/client/Chronometer.class';original=chronometer.read_bytes();chronometer.write_bytes(clock_provider(original))
 run=[str(jdk/'java.exe'),'-cp',cp,'fixture.Fixture',str(OUT/'inputs.json'),str(OUT/'oracle.json')]
 r=subprocess.run(run,cwd=ROOT,capture_output=True,text=True,encoding='utf-8',errors='replace');(OUT/'run.log').write_text(r.stdout+r.stderr,encoding='utf-8');print(r.stdout+r.stderr,end='');commands.append(dict(name='run',exitCode=r.returncode))
 if r.returncode:raise SystemExit(r.returncode)
 report=dict(cases=len(vectors),commands=commands,unchangedOriginalKotlin=all(p.read_bytes()==(work/p.name).read_bytes()for p in originals),sources={str(p.relative_to(ROOT)):hashlib.sha256(p.read_bytes()).hexdigest()for p in originals},clockProvider=dict(originalBytecodeSha256=hashlib.sha256(original).hexdigest(),providerBytecodeSha256=hashlib.sha256(chronometer.read_bytes()).hexdigest(),onlySystemClassConstantReplaced=True),oracleSha256=hashlib.sha256((OUT/'oracle.json').read_bytes()).hexdigest(),boundaries=['unchanged original module and Chronometer source, original Chronometer methods with controlled System clock class constant','event registration, module lifecycle, packet/player scalar storage, notifications and jump recording are dependency providers','not a complete original client execution'])
 (OUT/'oracle-metadata.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf-8');print(json.dumps(report))
if __name__=='__main__':main()
