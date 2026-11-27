#!/usr/bin/env python3
"""Pin all 65,536 actual original Agent Mth float values from the baseline JVM."""
import hashlib,json,os,subprocess
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
def main():
 out=ROOT/'build/simulation-sine-table';out.mkdir(parents=True,exist_ok=True)
 java=ROOT.parent/'sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe';kotlin=ROOT.parent/'sulfur-shulker-client/.tools/gradle/gradle-9.9.2/lib';binary=ROOT/'research/minecraft-agent-common-1.0.0-20261120.103027-24.jar';source=ROOT/'upstream/Minecraft-Agent/common/src/main/kotlin/net/ccbluex/agent/math/Mth.kt'
 fixture=out/'ReadTable.java';fixture.write_text('''import java.nio.*;import java.nio.file.*;
class ReadTable{public static void main(String[]args)throws Exception{
var field=Class.forName("net.ccbluex.agent.math.Mth").getDeclaredField("SIN");field.setAccessible(true);var values=(float[])field.get(null);
var bytes=ByteBuffer.allocate(values.length*4).order(ByteOrder.LITTLE_ENDIAN);for(float v:values)bytes.putInt(Float.floatToRawIntBits(v));Files.write(Path.of(args[0]),bytes.array());System.out.println(values.length);
}}\n''',encoding='utf-8')
 target=ROOT/'native/liquidbounce-behaviors/src/navigation/simulation/sine-table.bin'
 result=subprocess.run([str(java),'--class-path',os.pathsep.join([str(binary),str(next(kotlin.glob('kotlin-stdlib-*.jar')))]),str(fixture),str(target)],capture_output=True,text=True,encoding='utf-8',errors='replace');(out/'run.log').write_text(result.stdout+result.stderr,encoding='utf-8');print(result.stdout+result.stderr,end='')
 passed=result.returncode==0 and target.stat().st_size==65536*4
 (out/'report.json').write_text(json.dumps(dict(passed=passed,count=65536,sourceSha256=hashlib.sha256(source.read_bytes()).hexdigest(),baselineJarSha256=hashlib.sha256(binary.read_bytes()).hexdigest(),tableSha256=hashlib.sha256(target.read_bytes()).hexdigest(),scope='actual private Mth.SIN array from pinned unmodified Agent artifact on JDK25; no Rust/libc recomputation'),indent=2)+'\n',encoding='utf-8');raise SystemExit(0 if passed else 1)
if __name__=='__main__':main()
