// SPDX-License-Identifier: GPL-3.0-or-later
use crate::packets::{PacketEvent,PacketListener};
use horizon_client::mod_api::ClientboundPlayPacket as Clientbound;
use std::sync::atomic::{AtomicBool,AtomicU64,Ordering};
/// Packet threads only record delivery. Native reads/registry work run on actor.
#[derive(Default)]pub struct NoClipPackets {running:AtomicBool,pending:AtomicU64}
impl NoClipPackets {
 pub fn set_running(&self,running:bool){self.running.store(running,Ordering::Release);}
 pub fn take_pending(&self)->u64{self.pending.swap(0,Ordering::AcqRel)}
}
impl PacketListener for NoClipPackets {
 fn incoming(&self,event:&mut PacketEvent<'_,Clientbound>){
  // Cancellation does not stop original EventManager listener dispatch.
  if event.context.original()&&self.running.load(Ordering::Acquire)&&matches!(event.packet,Clientbound::PlayerPosition(_)){self.pending.fetch_add(1,Ordering::Release);}
 }
}
