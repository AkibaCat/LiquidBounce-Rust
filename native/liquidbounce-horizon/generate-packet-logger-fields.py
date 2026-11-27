"""Generate direct, typed field reads from the pinned protocol declarations."""
from pathlib import Path
import re,json
ROOT=Path(__file__).resolve().parents[2]
SRC=ROOT/'upstream/Horizon/crates/horizon-protocol/src/packets'
OUT=ROOT/'native/liquidbounce-horizon/engine/src/packet_logger_fields.rs'
def body(text,at):
 depth=1;i=at
 while depth:
  i+=1
  if text[i]=='{':depth+=1
  elif text[i]=='}':depth-=1
 return text[at+1:i]
def fields(text):
 result={'Empty':[]}
 text=re.sub(r'//[^\n]*','',text)
 for m in re.finditer(r'packet!\(\s*(\w+)\s*\{',text):
  result[m[1]]=re.findall(r'(\w+)\s*:\s*[^;\n]+?\s*=>',body(text,m.end()-1))
 for m in re.finditer(r'pub struct (\w+)\s*\{',text):
  result[m[1]]=re.findall(r'\bpub\s+(\w+)\s*:',body(text,m.end()-1))
 for m in re.finditer(r'pub struct (\w+)\(pub [^;]+\);',text):result[m[1]]=['0']
 for m in re.finditer(r'move_entity!\((\w+),',text):result[m[1]]=['0']
 return result
play=(SRC/'play.rs').read_text(encoding='utf8')
local=fields(play)
imports={}
for f in [SRC/'configuration.rs',SRC/'play/commands.rs',SRC/'play/hud.rs',SRC/'play/advancements.rs']:
 imports.update(fields(f.read_text(encoding='utf8')))
all_fields={**imports,**local}
output=['// SPDX-License-Identifier: GPL-3.0-or-later\n// Generated from pinned protocol structs; values are read directly, never parsed from Debug.\nuse horizon_client::mod_api::{ClientboundPlayPacket as C,ServerboundPlayPacket as S};\nuse liquidbounce_behaviors::packet_logger::Field;\nfn field<T:std::fmt::Debug>(name:&str,value:&T)->Field{Field{name:name.into(),kind:std::any::type_name::<T>().into(),value:format!("{value:?}")}}\n']
ids=(SRC.parent/'ids.rs').read_text(encoding='utf8');at=ids.index('pub mod play {');ids=body(ids,ids.index('{',at))
for direction,const in [('clientbound','S2C_IDS'),('serverbound','C2S_IDS')]:
 at=ids.index('pub mod '+direction+' {');names=re.search(r'NAMES:.*?=\s*(&\[.*?\]);',body(ids,ids.index('{',at)),re.S)[1]
 names=re.findall(r'"([^"]+)"',names)
 output.append(f'pub const {const}:&[&str]=&'+json.dumps(['minecraft:'+s for s in names])+';\n')
counts=[]
for enum,alias in [('Clientbound','C'),('Serverbound','S')]:
 m=re.search(r'packet_enum!\('+enum+r',[^\{]+\{(.*?)\}\);',play,re.S)
 entries=re.findall(r'(\w+)\(([^)]+)\)\s*=\s*(\w+)',re.sub(r'//[^\n]*','',m[1]))
 output.append(f'pub fn {enum.lower()}(packet:&{alias})->(&\'static str,Vec<Field>){{match packet{{\n')
 for variant,ty,ident in entries:
  if '::' in ty:
   parts=ty.split('::');path=SRC/(parts[-2]+'.rs');table=fields(path.read_text(encoding='utf8'));fs=table[parts[-1]]
  else:fs=all_fields[ty]
  reads=','.join(f'field("{f}",&p.{f})' for f in fs)
  output.append(f'{alias}::{variant}(p)=>("minecraft:{ident.lower()}",vec![{reads}]),\n')
 const='S2C_IDS' if alias=='C'else'C2S_IDS'
 output.append(f'{alias}::Unknown(id,payload)=>({const}.get(*id as usize).copied().unwrap_or("minecraft:unknown"),vec![field("id",id),field("payload",payload)]),\n}}}}\n')
 counts.append(len(entries))
OUT.write_text(''.join(output),encoding='utf8')
print('Generated direct packet field readers',counts)
