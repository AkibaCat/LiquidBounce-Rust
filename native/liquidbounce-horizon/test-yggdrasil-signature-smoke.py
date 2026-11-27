#!/usr/bin/env python3
"""One focused actual patched unpackTextures consumer smoke."""
from pathlib import Path
import hashlib,importlib.util,json,subprocess,time
ROOT=Path(__file__).resolve().parents[2]
HERE=Path(__file__).resolve().parent
OUT=ROOT/'build/yggdrasil-signature-smoke-01'
OUT.mkdir(parents=True,exist_ok=False)
spec=importlib.util.spec_from_file_location('patch',HERE/'yggdrasil-signature-patch.py');patch=importlib.util.module_from_spec(spec);spec.loader.exec_module(patch)
source=(ROOT/'upstream/Horizon/crates/horizon-client-render/src/skins.rs').read_text(encoding='utf8')
def once(source,old,new):
 if source.count(old)!=1:raise ValueError(old)
 return source.replace(old,new,1)
changed={}
patch.apply(lambda path,transform:changed.update({path:transform(source)}),once)
source=changed['crates/horizon-client-render/src/skins.rs']
enums=source[source.index('#[derive(Clone, Copy, Debug, PartialEq, Eq)]'):source.index('/// The `getTexture` endpoint')]
consumer=source[source.index('pub fn base64_decode'):source.index('/// The skin URL of a `minecraft:profile`')]
test='''
#[test]
fn actual_skin_policy_consumer_smoke() {
    use base64::Engine;
    let payload=base64::engine::general_purpose::STANDARD.encode(r#"{"textures":{"SKIN":{"url":"http://127.0.0.1:9000/skin","metadata":{"model":"classic"}},"CAPE":{"url":"https://custom.example/cape"}}}"#);
    let uris=["https://textures.minecraft.net/texture/{textureId}"];
    liquidbounce_yggdrasil_policy(false,true);
    assert_eq!(unpack(&payload,Signature::Invalid,&uris),Textures::default());
    liquidbounce_yggdrasil_policy(true,true);
    let accepted=unpack(&payload,Signature::Invalid,&uris);
    assert!(accepted.signed && accepted.slim);
    assert_eq!(accepted.cape.as_deref(),Some("https://custom.example/cape"));
    liquidbounce_yggdrasil_policy(true,false);
    let classic=unpack(&payload,Signature::Unsigned,&uris);
    assert!(classic.signed && !classic.slim);
    assert_eq!(unpack("broken",Signature::Signed,&uris),Textures::default());
    liquidbounce_yggdrasil_policy(false,true);
    assert_eq!(unpack(&payload,Signature::Signed,&uris),Textures::default());
}
'''
(OUT/'Cargo.toml').write_text('[package]\nname="yggdrasil-signature-smoke"\nversion="0.1.0"\nedition="2024"\n[dependencies]\nserde_json="1"\nbase64="0.22"\n[lib]\npath="lib.rs"\n',encoding='utf8')
(OUT/'lib.rs').write_text(enums+consumer+test,encoding='utf8')
start=time.time()
result=subprocess.run(['cargo','+stable','test','--lib'],cwd=OUT,text=True,encoding='utf8',stdout=subprocess.PIPE,stderr=subprocess.STDOUT)
(OUT/'cargo.log').write_text(result.stdout,encoding='utf8')
report={'passed':result.returncode==0,'tests':1,'scope':'actual patched skins::unpack only; no network download or full-module regression','elapsedSeconds':time.time()-start,'patchSha256':hashlib.sha256((HERE/'yggdrasil-signature-patch.py').read_bytes()).hexdigest()}
(OUT/'report.json').write_text(json.dumps(report,indent=2)+'\n',encoding='utf8')
print(result.stdout[-2500:]);print(json.dumps(report))
raise SystemExit(result.returncode)
