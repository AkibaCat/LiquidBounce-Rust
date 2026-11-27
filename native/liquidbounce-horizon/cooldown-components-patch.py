#!/usr/bin/env python3
"""Keep the network stack's USE_COOLDOWN for the real cooldown/render queries."""


def apply(edit, once):
    edit('crates/horizon-protocol/src/item.rs', lambda text: once(
        text, '    pub const CONSUMABLE: i32 = 24;\n',
        '    pub const CONSUMABLE: i32 = 24;\n    pub const USE_COOLDOWN: i32 = 26;\n'))

    def item_sync(text):
        text = once(text, '            // Read past, not decoded: the item keeps its prototype\'s value.\n', '''            // UseCooldown.STREAM_CODEC: float seconds, optional Identifier.
            // The same resolved saved component feeds client prediction and
            // the cooldown-group lookup used by the actual slot renderer.
            Component::Raw(bytes) if *kind == component::USE_COOLDOWN => {
                let mut reader = Reader::new(bytes);
                let decoded = (|| -> horizon_protocol::Result<Tag> {
                    let seconds = reader.f32()?;
                    let mut fields = vec![("seconds", Tag::Float(seconds.to_bits()))];
                    if reader.bool()? {
                        let group = horizon_protocol::buf::Ident::read(&mut reader)?;
                        fields.push(("cooldown_group", Tag::String(group.as_str().into())));
                    }
                    Ok(Tag::compound(fields))
                })();
                let Ok(tag) = decoded else { continue; };
                tag
            }
            // Read past, not decoded: the item keeps its prototype's value.
''')
        return once(text, '            if *id == component::DEATH_PROTECTION\n',
                    '            if matches!(*id, component::DEATH_PROTECTION | component::USE_COOLDOWN)\n')
    edit('crates/horizon-client/src/item_sync.rs', item_sync)

    def guest(text):
        text = once(text, '        "minecraft:tool" => write_tool(value, ids, &mut w)?,\n', '''        "minecraft:use_cooldown" => {
            let value = crate::cooldowns::UseCooldown::from_tag(value)?;
            w.f32(value.seconds);
            w.bool(value.group.is_some());
            if let Some(group) = value.group {
                w.string(&horizon_world::datafix::dynamic::namespaced(&group));
            }
        }
        "minecraft:tool" => write_tool(value, ids, &mut w)?,
''')
        return once(text, '            ("minecraft:tool", horizon_protocol::item::component::TOOL),\n', '''            ("minecraft:tool", horizon_protocol::item::component::TOOL),
            (
                "minecraft:use_cooldown",
                horizon_protocol::item::component::USE_COOLDOWN,
            ),
''')
    edit('crates/horizon-client/src/lan/guest.rs', guest)

    def cooldowns(text):
        return once(text, '''    stack
        .data()
        .and_then(|d| d.other("minecraft:use_cooldown"))
''', '''    if stack.data().is_some_and(|data| data.other("!minecraft:use_cooldown").is_some()) {
        return None;
    }
    stack
        .data()
        .and_then(|d| d.other("minecraft:use_cooldown"))
''')
    edit('crates/horizon-items/src/cooldowns.rs', cooldowns)
    return {'hook_names': [], 'helper_names': []}
