#!/usr/bin/env python3
"""Owned native TargetLock player/profile and isDeadOrDying readers."""
from pathlib import Path
HERE=Path(__file__).resolve().parent
def apply(edit,once):
    anchor='    /// Player FOV scale, after vanilla effects and before zoom and easing.\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda source:once(source,anchor,(HERE/'target-lock-host.rs').read_text(encoding='utf8')+'\n'+anchor))
    # Run after Hitbox's targeting helper. Preserve the common entity facts and
    # supply actual client-player/profile identity to the cached TagEntity event.
    anchor='        serde_json::json!({"kind":kind,"alive":alive,"invisible":invisible,"sleeping":sleeping,\n'
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda source:once(source,anchor,'        let lock_player: serde_json::Value = serde_json::from_str(&self.native_client_target_lock_player(entity)).unwrap_or(serde_json::Value::Null);\n'+anchor))
    edit('crates/horizon-client/src/game/native_hooks.rs',lambda source:once(source,'"within_border":within_border,"player_name":self.remote_player_name(entity as u32)',
         '"within_border":within_border,"player_name":lock_player["name"],"abstract_client_player":lock_player["name"].is_string()'))
    return {'hook_names':[],'helper_names':['client_target_lock_player','client_target_lock_dead']}
