// SPDX-License-Identifier: GPL-3.0-or-later
//! One InventoryManager for every inventoryPlan. Host calls belong to the
//! runtime adapter and never run while this actor is borrowed.
use liquidbounce_behaviors::{
    auto_tool::AutoTool,
    elytra_swap::{ElytraSwap, Item as ElytraSwapItem},
    inventory_manager::{
        Action, Context, CreativeStack, Geometry, Input, Manager, MenuState, Outcome, OwnerState,
        Position, Request, Requirements, Screen, Slot, Stack,
    },
    random::ThreadRandom,
    vomit::{self, Constraints, Vomit},
};
use serde_json::Value;

pub const AUTO_TOOL_OWNER: u64 = 0x4155_544f_544f_4f4c;
pub const VOMIT_OWNER: u64 = 0x0056_4f4d_4954;
pub const ELYTRA_SWAP_OWNER: u64 = 0x454c_5954_5241_5357;

fn field<'a>(value: &'a Value, name: &str) -> Result<&'a Value, String> {
    value
        .get(name)
        .ok_or_else(|| format!("inventory snapshot missing {name}"))
}
fn int(value: &Value, name: &str) -> Result<i32, String> {
    field(value, name)?
        .as_i64()
        .and_then(|n| i32::try_from(n).ok())
        .ok_or_else(|| format!("inventory snapshot invalid {name}"))
}
fn boolean(value: &Value, name: &str) -> Result<bool, String> {
    field(value, name)?
        .as_bool()
        .ok_or_else(|| format!("inventory snapshot invalid {name}"))
}
fn text(value: &Value, name: &str) -> Result<String, String> {
    field(value, name)?
        .as_str()
        .map(str::to_owned)
        .ok_or_else(|| format!("inventory snapshot invalid {name}"))
}
fn position(value: &Value) -> Result<Position, String> {
    let values = value
        .as_array()
        .filter(|a| a.len() == 2)
        .ok_or("invalid inventory position")?;
    Ok(Position {
        x: values[0].as_f64().ok_or("invalid inventory x")? as f32,
        y: values[1].as_f64().ok_or("invalid inventory y")? as f32,
    })
}
fn stack(value: &Value) -> Result<Stack, String> {
    Ok(Stack {
        components: text(value, "components")?,
        count: int(value, "count")?,
        wire: value
            .get("wire")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_owned(),
    })
}
fn stacks(value: &Value) -> Result<Vec<Stack>, String> {
    value
        .as_array()
        .ok_or("invalid inventory stacks")?
        .iter()
        .map(stack)
        .collect()
}
fn geometry(value: &Value) -> Result<Geometry, String> {
    Ok(Geometry {
        object: field(value, "object")?
            .as_u64()
            .ok_or("invalid menu object")?,
        slots: field(value, "slots")?
            .as_array()
            .ok_or("invalid menu slots")?
            .iter()
            .map(position)
            .collect::<Result<_, _>>()?,
    })
}
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub context: Context,
    pub block_registry_size: i32,
    pub elytra_swap_stacks: Vec<ElytraSwapItem>,
}
impl Snapshot {
    pub fn parse(payload: &str, combat: bool, move_cursor: bool) -> Result<Option<Self>, String> {
        let value: Value = serde_json::from_str(payload).map_err(|e| e.to_string())?;
        if value.is_null() {
            return Ok(None);
        }
        let menu = field(&value, "menu")?;
        let screen = field(&value, "screen")?;
        let screen = if screen.is_null() {
            None
        } else {
            Some(Screen {
                object: field(screen, "object")?
                    .as_u64()
                    .ok_or("invalid screen object")?,
                container_id: int(screen, "container")?,
                geometry: geometry(field(screen, "geometry")?)?,
                non_player_slots: field(screen, "nonPlayer")?
                    .as_array()
                    .ok_or("invalid container slots")?
                    .iter()
                    .map(|v| {
                        v.as_i64()
                            .and_then(|n| i32::try_from(n).ok())
                            .ok_or("invalid container slot".to_owned())
                    })
                    .collect::<Result<_, _>>()?,
            })
        };
        let cursor = field(&value, "visibleCursor")?;
        let movement = position(field(&value, "movement")?)?;
        Ok(Some(Self {
            block_registry_size: int(&value, "blockRegistrySize")?,
            elytra_swap_stacks: field(&value, "stacks")?
                .as_array()
                .ok_or("invalid ElytraSwap stacks")?
                .iter()
                .map(|stack| {
                    let item = field(stack, "elytraSwap")?;
                    Ok(ElytraSwapItem {
                        empty: boolean(item, "empty")?,
                        glider: boolean(item, "glider")?,
                        next_damage_will_break: boolean(item, "nextDamageWillBreak")?,
                        chest_armor: boolean(item, "chestArmor")?,
                    })
                })
                .collect::<Result<_, String>>()?,
            context: Context {
                now: field(&value, "now")?
                    .as_i64()
                    .ok_or("invalid inventory time")?,
                in_game: true,
                creative: boolean(&value, "creative")?,
                menu: MenuState {
                    container_id: int(menu, "container")?,
                    stacks: stacks(field(menu, "stacks")?)?,
                    carried: stack(field(menu, "carried")?)?,
                },
                inventory_menu: geometry(field(&value, "inventoryGeometry")?)?,
                screen,
                inventory_screen: boolean(&value, "inventoryScreen")?,
                container_screen: boolean(&value, "containerScreen")?,
                no_screen: boolean(&value, "noScreen")?,
                server_controlled_inventory: boolean(&value, "serverControlled")?,
                requirements: Requirements {
                    movement: [movement.x, movement.y],
                    jumping: boolean(&value, "jumping")?,
                    rotation_matches: boolean(&value, "rotationMatches")?,
                    using_item: boolean(&value, "using")?,
                    breaking: Some(boolean(&value, "breaking")?),
                    combat,
                },
                move_visible_cursor: move_cursor && !cursor.is_null(),
                visible_cursor: if cursor.is_null() {
                    Position::default()
                } else {
                    position(cursor)?
                },
                player_stacks: stacks(field(&value, "stacks")?)?,
            },
        }))
    }
    fn vomit(&self) -> vomit::Snapshot {
        let context = &self.context;
        let container = context
            .screen
            .as_ref()
            .filter(|_| context.container_screen)
            .map(|screen| vomit::Container {
                sync_id: screen.container_id,
                slot_count: screen.geometry.slots.len() as i32,
                nonempty_slots: screen
                    .non_player_slots
                    .iter()
                    .copied()
                    .filter(|slot| {
                        usize::try_from(*slot)
                            .ok()
                            .and_then(|slot| context.menu.stacks.get(slot))
                            .is_some_and(|s| !s.empty())
                    })
                    .collect(),
            });
        vomit::Snapshot {
            creative: context.creative,
            player_nonempty: std::array::from_fn(|slot| {
                context.player_stacks.get(slot).is_some_and(|s| !s.empty())
            }),
            container,
            block_registry_size: self.block_registry_size,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct InventoryActor {
    pub manager: Manager,
    next_plan: u64,
    auto_tool_request: Option<u64>,
    auto_tool_sample: Option<(u8, Stack)>,
    vomit_request: Option<(u64, bool)>,
    elytra_swap_request: Option<u64>,
    screen: Option<u64>,
}
impl InventoryActor {
    pub(crate) fn id(&mut self) -> u64 {
        self.next_plan = self.next_plan.wrapping_add(1);
        self.next_plan
    }
    pub fn observe_context(&mut self, context: &Context) {
        let screen = context.screen.as_ref().map(|screen| screen.object);
        if screen.is_some() && screen != self.screen {
            self.manager.opened();
        }
        self.screen = screen;
        self.manager.poll_timers(context.now);
    }
    pub fn begin_tick(&mut self, context: &Context) {
        self.observe_context(context);
        self.manager.begin_tick(context);
    }
    pub fn submit_auto_tool(
        &mut self,
        module: &mut AutoTool,
        running: bool,
        snapshot: &crate::auto_tool::Snapshot,
        context: &Context,
    ) {
        if self.auto_tool_request.is_some() || !module.is_inventory_considered(running) {
            return;
        }
        let Some(plan) = module.swaps.tick(
            &snapshot.stacks,
            snapshot.server_slot,
            module.swap_back_delay,
        ) else {
            return;
        };
        let id = self.id();
        let sample_slot = if plan.restoring {
            plan.action.hotbar
        } else {
            plan.action.source
        };
        self.auto_tool_sample = context
            .player_stacks
            .get(usize::from(sample_slot))
            .cloned()
            .map(|stack| (sample_slot, stack));
        self.manager.submit(Request::new(
            id,
            AUTO_TOOL_OWNER,
            0,
            vec![Action::Click {
                screen: None,
                slot: Slot::Player(plan.action.source),
                button: i32::from(plan.action.hotbar),
                input: Input::Swap,
            }],
            context,
            &self.manager,
        ));
        self.auto_tool_request = Some(id);
    }
    pub fn submit_vomit(
        &mut self,
        module: &mut Vomit,
        running: bool,
        snapshot: &Snapshot,
        random: &mut ThreadRandom,
    ) -> Result<(), String> {
        if self.vomit_request.is_some() || !running {
            return Ok(());
        }
        let Some(plan) = module
            .plan(&snapshot.vomit(), random)
            .map_err(str::to_owned)?
        else {
            return Ok(());
        };
        let creative = plan.creative;
        let slot = |slot| match slot {
            vomit::Slot::Player(slot) => Slot::Player(slot),
            vomit::Slot::Container(slot) => Slot::Container(slot),
        };
        let actions = plan
            .actions
            .into_iter()
            .map(|action| match action {
                vomit::Action::CreativeThrow { block_index, count } => Action::Creative {
                    stack: CreativeStack::Block {
                        registry_index: block_index,
                        count,
                    },
                    slot: None,
                },
                vomit::Action::CreativeFill {
                    block_index,
                    count,
                    slot: to,
                } => Action::Creative {
                    stack: CreativeStack::Block {
                        registry_index: block_index,
                        count,
                    },
                    slot: Some(slot(to)),
                },
                vomit::Action::Throw { screen, slot: from } => Action::Click {
                    screen: screen.and(snapshot.context.screen.clone()),
                    slot: slot(from),
                    button: 1,
                    input: Input::Throw,
                },
            })
            .collect();
        let id = self.id();
        self.manager.submit(Request::new(
            id,
            VOMIT_OWNER,
            0,
            actions,
            &snapshot.context,
            &self.manager,
        ));
        self.vomit_request = Some((id, creative));
        Ok(())
    }
    pub fn owners(
        &self,
        auto_tool: &AutoTool,
        auto_running: bool,
        tools: &crate::auto_tool::Snapshot,
        vomit: &Vomit,
        vomit_running: bool,
        creative: bool,
    ) -> [(u64, OwnerState); 2] {
        let c = &auto_tool.constraints;
        [
            (
                AUTO_TOOL_OWNER,
                OwnerState {
                    running: auto_tool.is_inventory_considered(auto_running),
                    guards_pass: auto_tool
                        .swaps
                        .guards_hold(&tools.stacks, auto_tool.swap_back_delay),
                    constraints: Constraints {
                        start_delay: c.start_delay,
                        click_delay: c.click_delay,
                        close_delay: c.close_delay,
                        miss_chance: c.miss_chance,
                        slot_multiplier: c.slot_multiplier,
                        millisecond_delays: c.millisecond_delays,
                        requirements: c.requirements,
                    },
                },
            ),
            (
                VOMIT_OWNER,
                OwnerState {
                    running: vomit_running,
                    guards_pass: self
                        .vomit_request
                        .is_some_and(|(_, capture)| capture == creative),
                    constraints: vomit.constraints.clone(),
                },
            ),
        ]
    }
    /// Returns true only for the plan's immediate enabled=false tail. A real
    /// pickup chain retains its owner until the Completed outcome is consumed.
    pub fn submit_elytra_swap(
        &mut self,
        module: &mut ElytraSwap,
        running: bool,
        snapshot: &Snapshot,
    ) -> Result<bool, String> {
        if self.elytra_swap_request.is_some() || !running {
            return Ok(false);
        }
        let stacks = &snapshot.elytra_swap_stacks;
        if stacks.len() != 41 {
            return Err("ElytraSwap requires the complete player inventory".into());
        }
        let pickups = module.pickups(stacks[38], |slot| stacks[usize::from(slot)]);
        if pickups.is_empty() {
            return Ok(true);
        }
        let id = self.id();
        self.manager.submit(Request::new(
            id,
            ELYTRA_SWAP_OWNER,
            0,
            pickups
                .into_iter()
                .map(|slot| Action::Click {
                    screen: None,
                    slot: Slot::Player(slot),
                    button: 0,
                    input: Input::Pickup,
                })
                .collect(),
            &snapshot.context,
            &self.manager,
        ));
        self.elytra_swap_request = Some(id);
        Ok(false)
    }
    pub fn elytra_swap_owner(&self, module: &ElytraSwap, running: bool) -> (u64, OwnerState) {
        (
            ELYTRA_SWAP_OWNER,
            OwnerState {
                running,
                guards_pass: true,
                constraints: module.constraints.clone(),
            },
        )
    }
    /// A cancellation abandons the continuation without reaching enabled=false.
    /// The direction captured by ModuleElytraSwap survives for the next plan.
    pub fn elytra_swap_outcome(&mut self, outcome: &Outcome) -> bool {
        let request = match outcome {
            Outcome::Completed { request }
            | Outcome::Cancelled { request }
            | Outcome::Performed { request, .. } => *request,
        };
        if self.elytra_swap_request != Some(request) || matches!(outcome, Outcome::Performed { .. })
        {
            return false;
        }
        self.elytra_swap_request = None;
        self.manager.end_plan(request);
        matches!(outcome, Outcome::Completed { .. })
    }
    fn auto_tool_tail_guard(&self, module: &AutoTool, context: &Context) -> bool {
        let Some(plan) = module.swaps.active.as_ref() else {
            return false;
        };
        let changed = self
            .auto_tool_sample
            .as_ref()
            .is_some_and(|(slot, sampled)| {
                context
                    .player_stacks
                    .get(usize::from(*slot))
                    .is_none_or(|current| {
                        !((current.empty() && sampled.empty())
                            || (current.count == sampled.count && current.same_item(sampled)))
                    })
            });
        if plan.restoring {
            (module.swaps.idle_ticks > module.swap_back_delay && module.swaps.requested.is_none())
                || changed
        } else {
            module.swaps.requested == Some(plan.action.source) || changed
        }
    }
    pub fn outcome(
        &mut self,
        outcome: &Outcome,
        auto_tool: &mut AutoTool,
        auto_running: bool,
        context: &Context,
    ) {
        let request = match outcome {
            Outcome::Performed { request, .. }
            | Outcome::Completed { request }
            | Outcome::Cancelled { request } => *request,
        };
        if self.auto_tool_request == Some(request) {
            match outcome {
                Outcome::Performed { .. } => {}
                Outcome::Completed { .. } => {
                    if auto_tool.is_inventory_considered(auto_running)
                        && self.auto_tool_tail_guard(auto_tool, context)
                    {
                        auto_tool.swaps.performed(true);
                    } else {
                        auto_tool.swaps.cancel_plan();
                    }
                    self.auto_tool_request = None;
                    self.auto_tool_sample = None;
                    self.manager.end_plan(request);
                }
                Outcome::Cancelled { .. } => {
                    auto_tool.swaps.cancel_plan();
                    self.auto_tool_request = None;
                    self.auto_tool_sample = None;
                    self.manager.end_plan(request);
                }
            }
        } else if self.vomit_request.is_some_and(|(id, _)| id == request)
            && !matches!(outcome, Outcome::Performed { .. })
        {
            self.vomit_request = None;
            self.manager.end_plan(request);
        }
    }
    pub fn observe(&mut self, payload: &str, now: i64) -> Result<(), String> {
        let value: Value = serde_json::from_str(payload).map_err(|e| e.to_string())?;
        let container = value.get("container").and_then(Value::as_i64).unwrap_or(0) as i32;
        match text(&value, "kind")?.as_str() {
            "mayOpen" => self.manager.may_open_container(now),
            "use" => self.manager.item_used(now),
            "click" => self.manager.sent_click(container),
            "open" => self.manager.container_opened(),
            "close" => self.manager.incoming_close(),
            "slot" => self.manager.resync_slot(
                container,
                int(&value, "slot")?,
                &stack(field(&value, "stack")?)?,
                now,
            ),
            "content" => {
                let records = field(&value, "items")?
                    .as_array()
                    .ok_or("invalid inventory content")?
                    .iter()
                    .map(|entry| stack(field(entry, "stack")?))
                    .collect::<Result<Vec<_>, _>>()?;
                self.manager.resync_content(container, &records, now);
            }
            other => return Err(format!("unknown inventory observation {other}")),
        }
        Ok(())
    }
    pub fn cancel_child(&mut self, module: &mut AutoTool) {
        self.manager.cancel_pending(AUTO_TOOL_OWNER);
        module.inventory_child_disabled();
    }
    pub fn cancel_vomit(&mut self) {
        self.manager.cancel_pending(VOMIT_OWNER);
    }
    pub fn cancel_elytra_swap(&mut self) {
        self.manager.cancel_pending(ELYTRA_SWAP_OWNER);
    }
    pub fn world_change(&mut self) {
        self.manager.world_change();
        self.screen = None;
    }
}
