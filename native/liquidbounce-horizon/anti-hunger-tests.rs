use super::*;
use anti_hunger::{AntiHunger, Context, Packet, Settings, Source};
use serde_json::{Value, json};
#[test]
fn unchanged_original_module_and_chronometer_match_all_traces() {
    let inputs: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/inputs.json"
    )))
    .unwrap();
    let oracle: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/oracle.json"
    )))
    .unwrap();
    for (case, (actions, expected)) in inputs
        .as_array()
        .unwrap()
        .iter()
        .zip(oracle.as_array().unwrap())
        .enumerate()
    {
        let mut state = AntiHunger::default();
        let mut running = true;
        let mut context = Context::default();
        for (step, (a, e)) in actions
            .as_array()
            .unwrap()
            .iter()
            .zip(expected.as_array().unwrap())
            .enumerate()
        {
            let mut trace = vec![];
            let mut sprint = None;
            let mut ground = None;
            let mut clock_index = 0;
            let clocks: Vec<i64> = a["clock"].as_array().map_or_else(
                || vec![1000, 1000],
                |v| {
                    v.iter()
                        .map(|v| v.as_str().unwrap().parse().unwrap())
                        .collect()
                },
            );
            let mut now = |trace: &mut Vec<String>| {
                let value = clocks[clock_index.min(clocks.len() - 1)];
                clock_index += 1;
                trace.push(format!("clock:{value}"));
                value
            };
            match a["op"].as_str().unwrap() {
                "settings" => {
                    state.settings = Settings {
                        no_sprint: a["noSprint"].as_bool().unwrap(),
                        while_swimming: a["whileSwimming"].as_bool().unwrap(),
                        keep_floating: a["floating"].as_bool().unwrap(),
                    }
                }
                "running" => running = a["value"].as_bool().unwrap(),
                "last" => state.last_notification = a["value"].as_str().unwrap().parse().unwrap(),
                "context" => {
                    let flags = a["flags"].as_u64().unwrap();
                    context = Context {
                        swimming: flags & 1 != 0,
                        passenger: flags & 2 != 0,
                        destroying: flags & 4 != 0,
                        in_water: flags & 8 != 0,
                        under_water: flags & 16 != 0,
                        fall_distance: f64::from_bits(a["fall"].as_str().unwrap().parse().unwrap()),
                    };
                }
                "sprint" => {
                    sprint = Some(state.sprint(
                        running,
                        context.swimming,
                        match a["source"].as_str().unwrap() {
                            "INPUT" => Source::Input,
                            "NETWORK" => Source::Network,
                            _ => Source::Movement,
                        },
                        a["value"].as_bool().unwrap(),
                    ))
                }
                "packet" => {
                    let kind = a["kind"].as_str().unwrap();
                    let packet = match kind {
                        "startSprint" | "stopSprint" => Some(Packet::PlayerCommand {
                            start_sprinting: kind == "startSprint",
                        }),
                        "startBreak" | "stopBreak" => Some(Packet::PlayerAction {
                            start_destroy: kind == "startBreak",
                        }),
                        _ => None,
                    };
                    if let Some(packet) = packet {
                        if let Some(warning) =
                            state.warning(running, context.swimming, packet, || now(&mut trace))
                        {
                            trace.push(format!(
                                "notification:AntiHunger:{}:INFO",
                                warning.message_key()
                            ));
                            state.notification_published(now(&mut trace));
                            if warning.jump() {
                                trace.push("jump".into());
                            }
                        }
                    } else if kind == "move" {
                        let mut value = a["ground"].as_bool().unwrap();
                        state.movement(running, context, &mut value);
                        ground = Some(value);
                    }
                }
                op => panic!("unknown {op}"),
            }
            let mut result = json!({"last":state.last_notification.to_string(),"trace":trace});
            if let Some(v) = sprint {
                result["sprint"] = json!(v);
            }
            if let Some(v) = ground {
                result["ground"] = json!(v);
            }
            assert_eq!(&result, e, "original case={case},step={step},action={a}");
        }
    }
}
#[test]
fn notification_queue_preserves_original_shape_and_language_at_emission() {
    use anti_hunger::Warning;
    let q = anti_hunger_notifications::NotificationQueue::default();
    q.publish(Warning::Sprint, "zh_cn".into());
    q.publish(Warning::Break, "en_us".into());
    let rows = q.drain(&|language, key| format!("{language}:{key}"));
    assert_eq!(
        rows,
        vec![
            (
                "notification".into(),
                json!({"title":"AntiHunger","message":"zh_cn:liquidbounce.module.antiHunger.messages.warnSprint","severity":"INFO"})
            ),
            (
                "notification".into(),
                json!({"title":"AntiHunger","message":"en_us:liquidbounce.module.antiHunger.messages.warnBreak","severity":"INFO"})
            )
        ]
    );
    assert!(q.drain(&|_, _| panic!("empty queue")).is_empty());
}
