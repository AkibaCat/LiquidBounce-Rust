#!/usr/bin/env python3
"""Hold a real pointer on registered widget IDs through the official socket."""


def apply(edit, once):
    def transform(text):
        text = once(text, 'use winit::event::MouseButton;', '''use winit::{
    dpi::PhysicalPosition,
    event::{DeviceId, ElementState, MouseButton, WindowEvent},
};''')
        text = once(text, '    "set ID VALUE                   move a slider to VALUE (0..1), as a click on its track",', '''    "set ID VALUE                   move a slider to VALUE (0..1), as a click on its track",
    "pointer ID move|down|up [left|right|middle]  move or hold the pointer at a registered widget",
    "pointer release               release pointer buttons held through window events",''')
        text = once(text, 'if !matches!(words[0], "ui" | "click" | "set") && !first_frame {',
                    'if !matches!(words[0], "ui" | "click" | "set" | "pointer") && !first_frame {')
        text = once(text, '        match words[0] {\n', '''        match words[0] {
            "pointer" => {
                let window_id = self.window.as_ref().ok_or("no window yet")?.id();
                let device_id = DeviceId::dummy();
                if words.get(1) == Some(&"release") {
                    let held: Vec<_> = self.physical_buttons.iter().copied().collect();
                    for button in held {
                        self.handle_window_event(event_loop, window_id,
                            WindowEvent::MouseInput { device_id, state: ElementState::Released, button },
                            super::input_trace::InputOrigin::Agent);
                    }
                    self.ui_wait.clicked();
                    return Ok(Handled::Reply(json!({"ok":true,"released":true})));
                }
                let node = self.find_node(id()?)?;
                let action = *words.get(2).ok_or("pointer needs move, down or up")?;
                let state = match action {
                    "move" => None,
                    "down" => Some(ElementState::Pressed),
                    "up" => Some(ElementState::Released),
                    _ => return Err("pointer needs move, down or up".into()),
                };
                if !node.enabled && action != "up" {
                    return Err(format!("widget {:?} is disabled", node.id).into());
                }
                let button = match words.get(3).copied().unwrap_or("left") {
                    "left" => MouseButton::Left,
                    "right" => MouseButton::Right,
                    "middle" => MouseButton::Middle,
                    _ => return Err("pointer button must be left, right or middle".into()),
                };
                let scale = self.renderer.as_ref()
                    .map(|r| horizon_client::hud::HudBatch::new([r.config.width,r.config.height]).scale)
                    .ok_or("no renderer yet")?;
                let position = PhysicalPosition::new(
                    f64::from((node.rect[0] + node.rect[2] / 2.0) * scale),
                    f64::from((node.rect[1] + node.rect[3] / 2.0) * scale));
                self.handle_window_event(event_loop, window_id,
                    WindowEvent::CursorMoved { device_id, position },
                    super::input_trace::InputOrigin::Agent);
                if let Some(state) = state {
                    self.handle_window_event(event_loop, window_id,
                        WindowEvent::MouseInput { device_id, state, button },
                        super::input_trace::InputOrigin::Agent);
                }
                self.ui_wait.clicked();
                Ok(Handled::Reply(json!({"ok":true,"pointer":node.id,"action":action,"screen":self.screen_label()})))
            }
''')
        return text

    edit('crates/horizon-client/src/game/ui_control.rs', transform)
    return {'hook_names': [], 'helper_names': []}
