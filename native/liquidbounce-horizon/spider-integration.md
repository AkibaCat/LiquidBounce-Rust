# Spider integration contract

Root-owned shared wiring:

1. behaviors `pub mod spider`; engine `mod spider_runtime` behind behavior-hooks;
   BehaviorModules `spider: liquidbounce_behaviors::spider::Spider`, Default.
   Register Spider Movement, key spider (source alias WallClimb).
2. Config: module/export SpiderSetting + serialize_spider; ModuleKind::Spider;
   SettingChange::Spider and RuntimeRequest::Spider, Copy setting. Parse with
   spider::parse_spider, serialize b.spider, setting.apply on actor.
3. WaitOwner::Spider; callback dispatch self.spider_callback(host,in_game,id).
   Run self.spider_tick(host) once in client_tick_pre after coroutine callbacks.
   CancelSequences Spider calls b.spider.cancel(). Mode setting's set_mode also
   cancels the old wait. The original requiresStop field persists across these.
4. Free generated hooks (no Host):
   client_spider_jump(&self,event:&mut ClientSpiderJumpEvent) calls
   self.spider_jump(event.horizontal_collision,&mut event.motion).
   client_spider_shape_active(&self,event:&mut ClientSpiderShapeActiveEvent)
   sets event.active |= self.spider_shape_active().
5. Add spider-patch.py after high-jump-patch.py and block-walk-patch.py. It adds
   actual live jump HEAD after HighJump, a scan hook after AirJump/BlockWalk's
   full-shape assignments, and three native Host helpers. Shape suppression
   honors original ShapeFlag. Whole collision boxes are decomposed, too-thin
   boxes dropped, x/z shrunk by 0.0001, then joined and optimized.

Polar queries actual registry running/mode on every shape through a free bridge,
so enabling/disabling/changing mode takes effect without waiting for GameTick.
The player's y/shift/ground policy is refreshed at client GameTick, every actual
local MotionWorld collision context query, local set_position and post tick.
The global shape policy matches the source singleton's cross-entity scope; it
must not be called while an engine state/behaviors guard is held across Host.
Full-client transformed shape/physics differential remains not_run.

Vulcan uses the common FIFO: two ticks, y=9.6599696; a new two-tick waiter, full
velocity=(0,0.0001,0); leaving collision later clears only y once. Exclusive
tickHandler drops game ticks while its sequence is suspended.

Focused validation: one Rust smoke passed across all three modes, two distinct
waiters, cancellation/persistent requiresStop and Polar height/thin-box rules.
All ten owned native edits consumed staged actual source exactly once in memory.
Actual generated-hook compilation remains the next root official Check.
