#!/usr/bin/env python3
"""Compile NoEntityInteract queries with actual Minecraft class/tag facts."""
from pathlib import Path
import json
import os
import runpy
import subprocess

HERE=Path(__file__).resolve().parent
ROOT=HERE.parent.parent
OUT=ROOT/"build/no-entity-interact-tests"


def main():
    ns=runpy.run_path(str(HERE/"reach-patch.py"))
    method=runpy.run_path(str(HERE/"test-movement-patch.py"))["method"]
    changed={}
    def replace_once(text,original,replacement):
        if text.count(original)!=1:raise ValueError(f"Expected one occurrence: {original[:100]}")
        return text.replace(original,replacement,1)
    def edit(path,transform):
        current=changed.get(path)
        if current is None:current=(ROOT/"upstream/Horizon"/path).read_text(encoding="utf-8")
        changed[path]=transform(current)
    fragment=ns["apply"](edit,replace_once)
    extra=runpy.run_path(str(HERE/"no-entity-interact-patch.py"))
    extra_fragment=extra["apply"](edit,replace_once)
    def take(path,name):return method(changed.get(path,(ROOT/"upstream/Horizon"/path).read_text(encoding="utf-8")),name).replace("pub(super)","pub(crate)")
    game="crates/horizon-client/src/game.rs"
    hand="crates/horizon-client/src/game/hand.rs"
    replacements={"PICKER":ns["PICKER"],"HOOKS":ns["HOOKS"]+extra["HOOKS"],
        "PLAYER_GUARD":take("crates/horizon-client/src/motion.rs","pub fn native_client_reach_queries_allowed("),
        "RAY_BOX":take(game,"fn ray_box("),"PICK_VIEW_VECTOR":take(game,"fn pick_view_vector("),"VIEW_VECTOR":take(game,"fn view_vector("),
        "EYE_POSITION":take(game,"fn eye_position("),"AIM":take(game,"fn aim("),"PICK":take(game,"fn pick(&self)"),
        "PICK_UNFILTERED":take(game,"fn pick_unfiltered("),"MOB_HIT":take(game,"fn mob_hit("),"MOB_WITHIN":take(game,"pub(super) fn mob_hit_within("),
        "ATTACK_REACH":take(hand,"fn attack_reach("),"REACH_PICK":take(hand,"fn reach_pick("),
        "INTERACTION_RANGE":take(hand,"pub(super) fn interaction_range("),"HELD_RANGE":take(hand,"pub(super) fn held_range_allows("),
        "ACTIVE_RANGE":take(hand,"pub(super) fn active_attack_range("),"ATTACK_RANGE_PICK":take(hand,"pub(super) fn attack_range_pick("),
        "CROSSHAIR_LIVING":take(hand,"pub(super) fn crosshair_living("),
        "HITTABLE_MOBS":take("crates/horizon-client/src/game/spear.rs","pub(super) fn hittable_mob_boxes("),
        "HITS_ALONG":take("crates/horizon-client/src/spear.rs","fn hits_along("),"HITS_IN_REACH":take("crates/horizon-client/src/spear.rs","pub fn hits_in_reach("),
    }
    specs={"PAINTING_HIT":("paintings","painting_hit"),"FRAME_HIT":("frames","frame_hit"),"BEFORE_BLOCK":("frames","before_block"),
        "STAND_HIT":("armor_stands","armor_stand_hit"),"CUSHION_HIT":("cushions","cushion_hit"),"BOAT_HIT":("boats","boat_hit"),
        "CART_HIT":("minecarts","cart_hit"),"INTERACTION_HIT":("interactions","interaction_hit"),"PROJECTILE_HIT":("deflect","redirectable_hit"),
        "DRAGON_HIT":("dragon","dragon_hit"),"REMOTE_DRAGON_HIT":("dragon","remote_dragon_part_hit"),"SPECIAL_TARGETS":("dragon","special_targets"),
        "KNOT_HIT":("remote_vehicles","remote_knot_hit"),"OBJECT_HIT":("remote_vehicles","remote_object_hit")}
    for key,(module,name) in specs.items():replacements[key]=take(f"crates/horizon-client/src/game/{module}.rs",f"fn {name}(").replace(f"fn {name}(",f"pub(crate) fn {name}(")
    source=(HERE/"reach-fixture.rs").read_text(encoding="utf-8")
    source=source.replace("extern crate self as horizon_world;","")
    source=source.replace('struct Game {', 'struct Game {options:Options,')
    source=source.replace('fn new()->Self {Self {', 'fn new()->Self {Self {options:Options {resources:std::path::PathBuf::from("__RESOURCE_ROOT__")},')
    source=source.replace("struct Stack {range:","struct Stack {item:String,range:").replace("Stack {range:",'Stack {item:"minecraft:diamond_sword".into(),range:')
    source=source.replace("fn reset() {", "fn reset() {NO_ENTITY.lock().unwrap().running=false;")
    event='        fn client_entity_pick(&self,_:&dyn Host,event:&mut ClientEntityPickEvent) {\n            let state=NO_ENTITY.lock().unwrap();event.allowed=event.allowed && state.module.test(state.running,event.main_hand_item);\n        }\n'
    source=source.replace("        fn client_rotation_aim(",event+"        fn client_rotation_aim(")
    source+='\n'+(HERE/"no-entity-interact-fixture.rs").read_text(encoding="utf-8")
    source=source.replace("__NO_ENTITY_PATH__",(ROOT/"native/liquidbounce-behaviors/src/no_entity_interact.rs").as_posix()).replace("__RESOURCE_ROOT__",(ROOT/"build/resources/26.3").as_posix())
    for key,value in replacements.items():source=source.replace(f"__{key}__",value)
    source=source.replace("super::spear::", "crate::spear::").replace("super::view_vector(", "crate::view_vector(")
    source=source.replace("__REACH_PATH__",(ROOT/"native/liquidbounce-behaviors/src/reach.rs").as_posix()).replace("__VIEW_DIRECTION_PATH__",(ROOT/"upstream/Horizon/crates/horizon-world/src/view_direction.rs").as_posix())
    OUT.mkdir(parents=True,exist_ok=True)
    facts=ROOT/"build/no-interact-oracle/facts.json"
    if not facts.is_file():subprocess.run(["python",str(HERE/"collect-no-interact-facts.py")],check=True)
    (OUT/"no-interact-facts.json").write_bytes(facts.read_bytes())
    java=ROOT.parent/"sulfur-shulker-client/.tools/java/jdk-25.0.5.1/bin/java.exe"
    cache=Path(os.environ["USERPROFILE"])/".gradle/caches"
    minecraft=cache/"fabric-loom/minecraftMaven/net/minecraft/minecraft-merged-deobf/26.3/minecraft-merged-deobf-26.3.jar"
    libraries=[str(minecraft)]
    for group in ["com.mojang","com.google.guava","com.google.code.gson","it.unimi.dsi","org.slf4j","org.joml","io.netty","org.apache.logging.log4j","org.apache.commons"]:
        libraries.extend(str(p) for p in (cache/"modules-2/files-2.1"/group).rglob("*.jar"))
    oracle=subprocess.run([str(java),"--class-path",os.pathsep.join(libraries),str(HERE/"ReachReference.java")],capture_output=True,text=True)
    if oracle.returncode:
        raise SystemExit(f"Reach Java reference failed ({oracle.returncode}):\n{oracle.stderr}")
    vectors=json.loads(oracle.stdout)
    (OUT/"java-vectors.json").write_text(json.dumps(vectors,indent=2)+"\n",encoding="utf-8")
    api_root=OUT/"api-source";api_game=api_root/"crates/horizon-client/src/game";api_game.mkdir(parents=True,exist_ok=True)
    (api_root/"tools/native-hook-fragments").mkdir(parents=True,exist_ok=True)
    (api_root/"crates/horizon-client/src/mod_api.rs").write_text("",encoding="utf-8")
    (api_game/"native_hooks.rs").write_text("impl Game<'_> {\n"+ns["HOOKS"]+extra["HOOKS"]+"\n}\n",encoding="utf-8")
    catalog={"helpers":[{"name":name,"target":f"horizon_client::game::native_hooks::Game<'_>::native_{name}"} for name in extra["HELPER_NAMES"]],"hooks":[{"name":name,"target":f"horizon_client::game::native_hooks::Game<'_>::native_{name}","mod_api":{"contract":"listen"}} for name in ns["HOOK_NAMES"]+extra["HOOK_NAMES"]]}
    (api_root/"native-hooks.json").write_text(json.dumps(catalog,indent=2)+"\n",encoding="utf-8")
    (OUT/"build.rs").write_text('fn main() {\nlet root=std::path::PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap()).join("api-source");\nlet out=std::path::PathBuf::from(std::env::var_os("OUT_DIR").unwrap());\nhorizon_mod_api_gen::generate_test_hooks(&root,&out).unwrap();\n}\n',encoding="utf-8")
    manifest='''[package]
name="liquidbounce-no-entity-interact-tests"
version="0.1.0"
edition="2024"
[dependencies]
horizon-core={path="__CORE__"}
serde_json="1"
libm="0.2"
horizon-world={path="__WORLD__"}
horizon-items={path="__ITEMS__"}
[build-dependencies]
horizon-mod-api-gen={path="__GENERATOR__"}
'''.replace("__CORE__",(ROOT/"upstream/Horizon/crates/horizon-core").as_posix()).replace("__WORLD__",(ROOT/"upstream/Horizon/crates/horizon-world").as_posix()).replace("__ITEMS__",(ROOT/"upstream/Horizon/crates/horizon-items").as_posix()).replace("__GENERATOR__",(ROOT/"upstream/Horizon/crates/horizon-mod-api-gen").as_posix())
    (OUT/"Cargo.toml").write_text(manifest,encoding="utf-8");(OUT/"src").mkdir(exist_ok=True);(OUT/"src/lib.rs").write_text(source,encoding="utf-8")
    result=subprocess.run(["cargo","+stable","test","--manifest-path",str(OUT/"Cargo.toml")],cwd=ROOT)
    evidence={"pristinePaths":list(changed),"hookFragment":fragment,"noEntityHookFragment":extra_fragment,"exitCode":result.returncode,"productionMethods":list(replacements),
        "apiGenerator":"horizon_mod_api_gen::generate_test_hooks","entityFilterOracle":"actual MC26.3 DefaultedMappedRegistry + Kotlin ComparisonsKt + Fastutil ObjectRBTreeSet + actual ArmorStand instance","scope":"production_all_twelve_pickable_categories;actual_Reach_early_return_precedence;actual_Items_tag_defaults;generated_hook_and_helper_catalog;full_client_differential_not_run"}
    (OUT/"result.json").write_text(json.dumps(evidence,indent=2)+"\n",encoding="utf-8")
    raise SystemExit(result.returncode)


if __name__=="__main__":main()
