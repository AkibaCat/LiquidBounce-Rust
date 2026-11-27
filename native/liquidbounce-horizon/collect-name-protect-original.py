#!/usr/bin/env python3
"""Prepare unchanged pinned NameGenerator source oracle; --run explicitly compiles/runs Kotlin."""
from pathlib import Path
import argparse,hashlib,json,os,shutil,subprocess,time
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]


def prepare(out):
    out.mkdir(parents=True,exist_ok=True);fixture=out/'fixture';fixture.mkdir(exist_ok=True);classes=out/'classes';classes.mkdir(exist_ok=True)
    source=ROOT/'upstream/LiquidBounce/src/main/kotlin/net/ccbluex/liquidbounce/utils/client/NameGenerator.kt';shutil.copyfile(source,fixture/source.name)
    (fixture/'LiquidBounce.kt').write_text('package net.ccbluex.liquidbounce\nobject LiquidBounce\n',encoding='utf8')
    (fixture/'PairExtensions.kt').write_text('''package net.ccbluex.fastutil
import it.unimi.dsi.fastutil.ints.IntCharPair
operator fun IntCharPair.component1():Int=firstInt()
operator fun IntCharPair.component2():Char=secondChar()
''',encoding='utf8')
    (fixture/'SubList.kt').write_text('''package net.ccbluex.liquidbounce.utils.kotlin
inline fun <T> List<T>.subList(fromIndex:Int):List<T>{return this.subList(fromIndex,this.size)}
''',encoding='utf8')
    (fixture/'Main.kt').write_text('''import net.ccbluex.liquidbounce.utils.client.randomUsername
import kotlin.random.Random
import com.google.gson.Gson
import com.google.gson.JsonParser
import java.io.File
import java.nio.ByteBuffer
import java.security.MessageDigest
fun main(args:Array<String>){
 val rows=JsonParser.parseString(File(args[0]).readText()).asJsonArray.map {entry->
  val name=entry.asString
  val seed=ByteBuffer.wrap(MessageDigest.getInstance("MD5").digest(name.toByteArray(Charsets.UTF_8))).long
  mapOf("name" to name,"replacement" to randomUsername(16,Random(seed)))
 }
 File(args[1]).writeText(Gson().toJson(rows))
}
''',encoding='utf8')
    assets=ROOT/'upstream/LiquidBounce/src/main/resources/resources/liquidbounce/data/usernames'
    destination=classes/'resources/liquidbounce/data/usernames';destination.mkdir(parents=True,exist_ok=True)
    for name in ['adjectives.txt','animals.txt']:shutil.copyfile(assets/name,destination/name)
    names=['','Alice','Bob','Notch','Steve','Alex','玩家😀','😀','_','a','Ab','abcd','abcdefghijklmnopqrst','😀😀😀😀😀😀😀😀😀😀','测试玩家','CaseSensitive','casesensitive']+[f'Player_{i:04d}'for i in range(512)]
    (out/'inputs.json').write_text(json.dumps(names,ensure_ascii=False),encoding='utf8')
    java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';libs=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib'
    full_fastutil=ROOT/'build/resource-runtime/libraries/it/unimi/dsi/fastutil/8.5.18/fastutil-8.5.18.jar'
    deps=[next(libs.glob(n))for n in ['kotlin-stdlib-*.jar','guava-*.jar','gson-*.jar']]+[full_fastutil]
    cp=os.pathsep.join(map(str,[classes,*deps]))
    commands=[('compile',[str(java),'-cp',str(libs/'*'),'org.jetbrains.kotlin.cli.jvm.K2JVMCompiler','-no-stdlib','-no-reflect','-classpath',cp,'-d',str(classes),*map(str,sorted(fixture.glob('*.kt')))]),('oracle',[str(java),'-cp',cp,'MainKt',str(out/'inputs.json'),str(out/'oracle.json')])]
    report={'status':'prepared-no-compiler-invoked','vectors':len(names),'unchangedSource':source.read_bytes()==(fixture/source.name).read_bytes(),'scope':'Byte-identical NameGenerator.kt and resource word lists with real Kotlin Random(Long), Guava IntMath and full fastutil. Only LiquidBounce resource anchor and pair/subList extension declarations supplied; entropy is standard Java MD5 UTF-8 and big-endian ByteBuffer.long, matching pinned source. No engine/game claim.','sources':{p.relative_to(ROOT).as_posix():hashlib.sha256(p.read_bytes()).hexdigest()for p in [source,assets/'adjectives.txt',assets/'animals.txt']},'commands':commands}
    (out/'preparation.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');return report,commands


def main():
    parser=argparse.ArgumentParser(description=__doc__);parser.add_argument('--out',required=True);parser.add_argument('--run',action='store_true');args=parser.parse_args();out=ROOT/args.out
    report,commands=prepare(out)
    if not args.run:print(f'NameProtect original-source oracle prepared: {out}; no compiler invoked');return
    results=[]
    for name,cmd in commands:
        start=time.monotonic();r=subprocess.run(cmd,cwd=ROOT,capture_output=True,text=True,encoding='utf8',errors='replace');(out/f'{name}.log').write_text(r.stdout+r.stderr,encoding='utf8');results.append({'name':name,'exitCode':r.returncode,'seconds':round(time.monotonic()-start,3)})
        if r.returncode:break
    report['status']='executed';report['passed']=len(results)==2 and all(c['exitCode']==0 for c in results);report['results']=results
    (out/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8');print(json.dumps({'passed':report['passed'],'vectors':report['vectors'],'report':str(out/'report.json')}));raise SystemExit(0 if report['passed']else 1)


if __name__=='__main__':main()
