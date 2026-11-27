"""GodBridge/Breezily and arbitrary-eye crosshair; after source Normal/Expand."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
ROOT=HERE.parents[1]
def apply(edit,once):
 for target,source in [('scaffold_special_rules.rs',ROOT/'native/liquidbounce-behaviors/src/scaffold_special_techniques.rs'),('scaffold_special_techniques.rs',HERE/'scaffold-special-techniques-native.rs')]:
  edit('crates/horizon-client/src/game/'+target,lambda _,source=source:source.read_text(encoding='utf8'))
 edit('crates/horizon-client/src/game.rs',lambda s:once(s,'mod scaffold_techniques;','mod scaffold_techniques;\nmod scaffold_special_rules;\nmod scaffold_special_techniques;'))
 anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
 edit('crates/horizon-client/src/game/native_hooks.rs',lambda s:once(s,anchor,(HERE/'scaffold-special-techniques-host.rs').read_text(encoding='utf8')+'\n'+anchor))
 def techniques(s):
  s=once(s,'if !matches!(technique, "Normal" | "Expand")','if !matches!(technique, "Normal" | "Expand" | "GodBridge" | "Breezily")')
  s=once(s,'                } else {\n                    let length =','                } else if technique == "Expand" {\n                    let length =')
  a='''                };
                Some(
                    json!({"target":found.as_ref().map(write_target),"heightState":height.write()})'''
  b='''                } else {
                    let mut options=placement::Options::center(&stack,location);
                    options.offsets=placement::offsets(1);
                    options.priority=placement::Priority::Position(predicted);
                    let block=height.target(predicted.map(|v|v.floor()as i32));
                    placement::find(self,block,&mut options)
                };
                Some(
                    json!({"target":found.as_ref().map(write_target),"heightState":height.write()})'''
  s=once(s,a,b)
  # These techniques use ScaffoldTechnique's plain verifyClick default.
  return once(s,'let fallback = technique == "Expand" || request["shouldGoDown"].as_bool()?;','let fallback = technique == "Expand" || technique == "Normal" && request["shouldGoDown"].as_bool()?;')
 edit('crates/horizon-client/src/game/scaffold_techniques.rs',techniques)
 def placement(s):
  a='''    let view = horizon_world::view_direction::view_vector(rotation[1], rotation[0]);
    let d = [view.x, view.y, view.z];'''
  start=s.index('pub(super) fn trace_world_click(')
  prefix,body=s[:start],s[start:]
  body=once(body,a,'''    trace_from_point(game,start,rotation,range)
}
pub(super) fn trace_from_point(game:&Game<'_>,start:[f64;3],rotation:[f32;2],range:f64)->Option<Click>{
'''+a)
  return prefix+body
 edit('crates/horizon-client/src/game/block_placement.rs',placement)
 return {'hook_names':[],'helper_names':['client_scaffold_special_technique']}
