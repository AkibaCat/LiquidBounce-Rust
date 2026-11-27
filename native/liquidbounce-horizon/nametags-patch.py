"""Native Nametags renderer, snapshot seam and vanilla-name suppression."""
from pathlib import Path

HERE = Path(__file__).resolve().parent


def apply(edit, once):
    edit("crates/horizon-client/src/game/nametags.rs", lambda _: (HERE / "nametags-game.rs").read_text(encoding="utf8"))
    edit("crates/horizon-client/src/game/nametags_snapshot.rs", lambda _: (HERE / "nametags-snapshot.rs").read_text(encoding="utf8"))

    def game(s):
        s = once(s, "mod bed_plates;", "mod bed_plates;\nmod nametags;\nmod nametags_snapshot;")
        s = once(s, "    client_bed_plates:std::cell::RefCell<bed_plates::State>,", "    client_bed_plates:std::cell::RefCell<bed_plates::State>,\n    client_nametags:std::cell::RefCell<nametags::State>,")
        s = once(s, "            client_bed_plates:Default::default(),", "            client_bed_plates:Default::default(),\n            client_nametags:Default::default(),")
        s = once(s, "            self.client_item_tags_overlay(&mut batch);", "            self.client_nametags_overlay(&mut batch);\n            self.client_item_tags_overlay(&mut batch);")
        return s

    edit("crates/horizon-client/src/game.rs", game)

    def hooks(s):
        anchor = "    pub(super) fn native_client_item_tags("
        s = once(s, anchor, "    pub(super) fn native_client_nametags(&self,input:&str,output:&mut String){if self.native_window(){mod_api::dispatch_client_nametags(self,mod_api::mods(),input,output);}}\n\n" + anchor)
        s = once(s, "        self.client_bed_plates_tick();", "        self.client_bed_plates_tick();\n        self.client_nametags_tick();")
        s = once(s, "        self.client_bed_plates_world_change();", "        self.client_bed_plates_world_change();\n        *self.client_nametags.borrow_mut()=Default::default();")
        # Keep custom labels supplied by another hook, while suppressing ordinary vanilla labels.
        needle = "        mod_api::dispatch_render_name_tag(\n            self,\n            mod_api::mods(),\n            entity_id,\n            kind,\n            vanilla_visible,\n            visible,\n            text,\n        );"
        replacement = needle + "\n        if !self.client_nametags_vanilla(entity_id,text.is_some()){*visible=false;}"
        return once(s, needle, replacement)

    edit("crates/horizon-client/src/game/native_hooks.rs", hooks)
    return {"hook_names": ["client_nametags"], "helper_names": []}

