// SPDX-License-Identifier: GPL-3.0-or-later
// Uses actual generated Horizon item sync, item cooldowns and packet codecs.
use horizon_client_lib::{cooldowns, item_sync, items::ItemStack, lan::guest};
use horizon_protocol::{
    buf::{Codec, Reader, Writer},
    item::{Component, Stack, Template, component},
    registry::{Builtin, Dynamic, Ids},
};
use horizon_world::nbt::Tag;
use serde_json::{Value, json};
use std::{collections::HashMap, sync::Arc};

fn main() {
    let rows: Vec<Value> = serde_json::from_str(
        &std::fs::read_to_string(std::env::args_os().nth(1).unwrap()).unwrap(),
    )
    .unwrap();
    let mut names = vec![String::new(); component::USE_COOLDOWN as usize + 1];
    names[component::USE_COOLDOWN as usize] = "minecraft:use_cooldown".into();
    let builtin = Builtin {
        registries: HashMap::from([
            (
                "minecraft:item".into(),
                Ids::from_names(vec!["minecraft:snowball".into()]),
            ),
            (
                "minecraft:data_component_type".into(),
                Ids::from_names(names),
            ),
        ]),
    };
    let dynamic = Dynamic::default();
    let default = cooldowns::UseCooldown {
        seconds: 30.0,
        group: Some(Arc::from("liquidbounce:default")),
    };
    let mut checked = 0;
    for row in rows {
        let hex = row["bytes"].as_str().unwrap();
        let bytes: Vec<u8> = (0..hex.len())
            .step_by(2)
            .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
            .collect();
        let mut saved = ItemStack::new("minecraft:snowball", 16);
        let mut fields = vec![(
            "seconds",
            Tag::Float(row["secondsBits"].as_i64().unwrap() as u32),
        )];
        if let Some(group) = row["savedGroup"].as_str() {
            fields.push(("cooldown_group", Tag::String(group.into())));
        }
        saved
            .data_mut()
            .set_other("minecraft:use_cooldown", Some(Tag::compound(fields)));
        let encoded = guest::stack_to_network(|_| 0, |_, _| None, &Some(saved)).unwrap();
        assert_eq!(
            encoded.component(component::USE_COOLDOWN),
            Some(&Component::Raw(bytes))
        );
        // Exercise the actual integrated server encoder before client decode.
        let stack = encoded;
        let mut writer = Writer::new();
        Template::write(&stack, &mut writer);
        let decoded = Template::read(&mut Reader::new(&writer.data)).unwrap();
        let local = item_sync::from_network(&decoded, &builtin, &dynamic);
        let value = cooldowns::component(&local, Some(&default)).unwrap();
        assert_eq!(
            value.seconds.to_bits() as i32,
            row["secondsBits"].as_i64().unwrap() as i32
        );
        assert_eq!(value.ticks(), row["ticks"].as_i64().unwrap() as i32);
        let expected = row["group"].as_str().unwrap_or("minecraft:snowball");
        assert_eq!(&*cooldowns::group(&local, Some(&default)), expected);
        let mut map = cooldowns::Cooldowns::default();
        map.add(cooldowns::group(&local, Some(&default)), value.ticks());
        if value.ticks() > 0 {
            assert!(map.is_on_cooldown(expected));
        }
        checked += 1;
    }
    let mut removed = Stack::new(0, 1);
    removed.removed.push(component::USE_COOLDOWN);
    let mut saved = ItemStack::new("minecraft:snowball", 1);
    saved.data_mut().set_other(
        "!minecraft:use_cooldown",
        Some(Tag::compound([] as [(&str, Tag); 0])),
    );
    let encoded = guest::stack_to_network(|_| 0, |_, _| None, &Some(saved)).unwrap();
    assert_eq!(encoded.removed, removed.removed);
    removed = encoded;
    let mut writer = Writer::new();
    Template::write(&removed, &mut writer);
    let removed = Template::read(&mut Reader::new(&writer.data)).unwrap();
    let local = item_sync::from_network(&removed, &builtin, &dynamic);
    assert!(cooldowns::component(&local, Some(&default)).is_none());
    assert_eq!(
        &*cooldowns::group(&local, Some(&default)),
        "minecraft:snowball"
    );
    let unpatched = item_sync::from_network(&Stack::new(0, 1), &builtin, &dynamic);
    assert_eq!(
        cooldowns::component(&unpatched, Some(&default)).unwrap(),
        default
    );
    assert_eq!(
        &*cooldowns::group(&unpatched, Some(&default)),
        "liquidbounce:default"
    );
    println!(
        "{}",
        json!({"passed":true,"actualMinecraftCodecRows":checked,"actualServerEncoder":true,"removalAndPrototypeFallback":true})
    );
}
