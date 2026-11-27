from pathlib import Path
import re,json,zipfile,hashlib
ROOT=Path(__file__).resolve().parents[2]
def main():
 table=(ROOT/'upstream/Horizon/crates/horizon-client-core/src/potions.rs').read_text(encoding='utf8').split('pub const EFFECTS:')[1].split('];')[0]
 ids=re.findall(r'(?:fx\("|id: ")([a-z_]+)"',table)
 jar=Path.home()/'.gradle/caches/fabric-loom/26.3/minecraft-client.jar'
 with zipfile.ZipFile(jar) as z:language=json.loads(z.read('assets/minecraft/lang/en_us.json'))
 assert len(ids)==len(set(ids))==40
 out='pub const EFFECTS: &[(&str,&str)] = &[\n'+''.join(f'    ({json.dumps(i)},{json.dumps(language.get("effect.minecraft."+i,"Unknown"))}),\n' for i in ids)+'];\n'
 (ROOT/'native/liquidbounce-behaviors/src/potion_spoof_effects.rs').write_text(out,encoding='utf8')
 print(json.dumps({'effects':len(ids),'client_sha256':hashlib.sha256(jar.read_bytes()).hexdigest()}))
if __name__=='__main__':main()
