# MiddleClickAction owned delivery

Reference: `ModuleMiddleClickAction.kt` at ae0eb396. Misc, aliases `FriendClicker`, `MiddleClickPearl`, key `middleClickAction`. Entire module is `doNotIncludeAlways`, exclude only from PUBLIC serializer exactly like TargetLock/GuiCloser.

Shared wiring (root owns):

- behaviors lib export `pub mod middle_click_action;`; `BehaviorModules.middle_click_action: MiddleClickAction` default.
- engine lib add `middle_click_action_runtime`; config add/re-export `MiddleClickActionSetting,serialize_middle_click_action`, ModuleKind / SettingChange / serialize / parse / actor setting apply.
- GameTick after FIRST_PRIORITY target-cache clearing: `self.middle_click_action_tick(host)` once with mutable Host. Native snapshot is outside all state locks because pick margin, shape, GhostHand hooks read engine state.
- CancelSequences for MiddleClickAction: `pearl_disable()` and pending_slot_resets.push(`crate::middle_click_action_runtime::PEARL_REQUESTER`). FriendClicker's clicked flag must persist. Mode-setting uses set_mode, which clears Pearl when leaving it but does not reset FriendClicker clicked.
- WorldChange/disconnect: `middle_click_action.world_change()` (only the active Pearl's flag resets); SilentHotbar's existing world cleanup handles its request.
- Export existing upstream `interaction_key` hook. Add `fn interaction_key(&self,host:&mut dyn Host,event:&mut horizon_client::mod_api::InteractionKeyEvent)`: if `event.button==2 && self.middle_click_action_cancel_pick(host)`, `*event.canceled=true`. Both real local and remote pick paths invoke it before any pick action. This hook must run on already-cancelled events too. No sound/swing change needed.
- Compose `middle-click-action-patch.py` after auto-break, hitbox/reach and ghost-hand. Add its two helper_names. Patch supplies player-only ProjectileUtil candidates with actual eye, real player rotation, actual pick margin, camera box/root and COLLIDER line of sight. Offhand-first pearl lookup matches Slots enum.

One focused standalone smoke: `middle-click-action-smoke.rs` covers held/release, offhand, missing-item retention, pitch suppression, world/disable and FriendClicker miss/blocked-target edges. Native generation/compile belongs to root's official check, not a full-client result.
