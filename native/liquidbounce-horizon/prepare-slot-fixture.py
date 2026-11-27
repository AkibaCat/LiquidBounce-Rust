#!/usr/bin/env python3
"""Compile actual patched Session and actor slot bodies with framed Pipe IO."""
from pathlib import Path
import hashlib
import importlib.util

HERE = Path(__file__).resolve().parent
PROJECT = HERE.parent.parent
changes = {}


def replace_once(text, before, after):
    if text.count(before) != 1:
        raise ValueError(f"Expected one anchor: {before[:100]}")
    return text.replace(before, after, 1)


def edit(name, transform):
    before = changes.get(name)
    if before is None:
        before = (PROJECT / "upstream/Horizon" / name).read_text(encoding="utf-8")
    changes[name] = transform(before)


for name in ["packet", "slot"]:
    spec = importlib.util.spec_from_file_location(name + "_patch", HERE / (name + "-patch.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    module.apply(edit, replace_once)


def function(text, signature):
    first = text.index(signature)
    opening = text.index("{", first)
    depth = 1
    end = opening + 1
    while depth:
        depth += (text[end] == "{") - (text[end] == "}")
        end += 1
    return text[first:end]


game = "crates/horizon-client/src/game/"
network = changes["crates/horizon-client/src/net.rs"]
remote = changes[game + "remote.rs"]
native = changes[game + "native_hooks.rs"]
methods = "\n".join(function(network, signature) for signature in ["    pub fn send(&self,", "    pub fn send_silently(&self,", "    fn send_with_path(&self,"])
template = (HERE / "packet-tests/transport-fixture-template.rs").read_text(encoding="utf-8")
output = "// Generated from exact transformed transport and actor slot sources.\n"
output += "// Source SHA256: " + hashlib.sha256("".join(changes[name] for name in sorted(changes)).encode()).hexdigest() + "\n"
output += "extern crate liquidbounce_horizon_packet_tests as horizon_client;\n"
output += template[:template.index("    #[test]")].replace("/* GENERATED_SESSION_METHODS */", methods).replace("/* GENERATED_RECEIVE_PLAY */", function(network, "fn receive_play("))
output += '''
use std::cell::Cell;
use play::{Clientbound as C, Serverbound as S, Clientbound as ClientboundPlayPacket};
use liquidbounce_horizon_packet_tests::{slots::*, slot_bridge::SlotPolicy};
type Result<T, E = String> = std::result::Result<T, E>;
struct Remote { session: Session, carried: Cell<Option<usize>> }
struct Player { server_copy: bool, replaying: bool }
impl Player { fn native_client_queries_allowed(&self) -> bool { !self.server_copy && !self.replaying } }
struct Game { selected: usize, player: Option<Player>, lan_acting_uuid: Option<u128>, window: bool, remote: Option<Remote>, read_only_calls: Cell<usize> }
impl Game {
    fn native_window(&self) -> bool { self.window }
    fn native_client_packet(&self, _: &C) { self.read_only_calls.set(self.read_only_calls.get() + 1); }
'''
for signature in [
    "    pub(super) fn native_client_slot_owner(",
    "    pub(super) fn native_client_real_selected_slot(",
    "    pub(super) fn client_hotbar_slot(",
    "    pub(super) fn native_client_serverside_slot(",
    "    pub(super) fn native_client_clientside_slot(",
    "    pub(super) fn native_client_select_slot_silently(",
    "    pub(super) fn native_client_reset_silent_slot(",
    "    pub(super) fn native_client_silent_slot_modified(",
    "    pub(super) fn native_client_silent_slot_modified_by(",
    "    pub(super) fn native_reject_slot_packet(",
]:
    output += function(native, signature) + "\n"
for signature in ["    pub(super) fn send(&self,", "    fn remote_send_carried(", "    pub(super) fn handle_packet(&mut self,"]:
    output += function(remote, signature) + "\n"
branch = function(remote, "            C::SetHeldSlot(p) =>")
output += "    #[allow(clippy::single_match, clippy::collapsible_match)]\n    fn handle_packet_direct(&mut self, packet: C) -> Result<()> {\n        match packet {\n" + branch + "\n            _ => {}\n        }\n        Ok(())\n    }\n"
for signature in ["    pub(super) fn native_client_disconnecting(", "    pub(super) fn native_client_level_load(", "    pub(super) fn native_client_level_unload(", "    pub(super) fn native_client_tick_pre("]:
    output += function(native, signature) + "\n"
output += "}\n"
output += '''
mod mod_api {
    use super::Game;
    pub fn mods() -> &'static [()] { &[] }
    pub fn dispatch_client_disconnecting(_: &Game, _: &[()]) {}
    pub fn dispatch_client_level_load(_: &Game, _: &[()], _: &str) {}
    pub fn dispatch_client_level_unload(_: &Game, _: &[()], _: &str) {}
    pub fn dispatch_client_tick_pre(_: &Game, _: &[()]) {}
}
'''
output += (HERE / "packet-tests/slot-fixture-tests.rs").read_text(encoding="utf-8")
output += "\n}\n"
destination = HERE / "packet-tests/tests/generated_slots.rs"
if not destination.exists() or destination.read_text(encoding="utf-8") != output:
    destination.write_text(output, encoding="utf-8", newline="\n")
print(destination)
