// SPDX-License-Identifier: GPL-3.0-or-later
//! Transport-facing packet extension, copied into the staged Horizon client lib.
//! This module owns no Game, registry, control socket or writer lock.

use horizon_protocol::packets::play::{Clientbound, Serverbound};
use std::cell::Cell;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::{Arc, OnceLock, RwLock};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TransferOrigin {
    Incoming,
    Outgoing,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Delivery {
    Original,
    Silent,
    IncomingReplay,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PacketContext {
    pub session: u64,
    pub origin: TransferOrigin,
    pub delivery: Delivery,
}

impl PacketContext {
    pub const fn original(self) -> bool {
        matches!(self.delivery, Delivery::Original)
    }
}

pub struct PacketEvent<'a, P> {
    pub context: PacketContext,
    pub packet: &'a mut P,
    cancelled: bool,
}

impl<'a, P> PacketEvent<'a, P> {
    pub fn new(context: PacketContext, packet: &'a mut P) -> Self {
        Self {
            context,
            packet,
            cancelled: false,
        }
    }

    pub fn cancel(&mut self) {
        self.cancelled = true;
    }

    pub const fn is_cancelled(&self) -> bool {
        self.cancelled
    }
}

/// Callbacks can run on the socket reader or game thread. They must not access
/// Game, perform blocking game-thread requests or dispatch with a registry lock.
pub trait PacketInterceptor: Send + Sync {
    fn incoming(&self, _event: &mut PacketEvent<'_, Clientbound>) {}
    fn outgoing(&self, _event: &mut PacketEvent<'_, Serverbound>) {}
    fn cancel_local_swing(&self) -> bool {
        false
    }
    fn bypass_pitch_limit(&self) -> bool {
        false
    }
}

type Interceptor = Arc<dyn PacketInterceptor>;
static INTERCEPTOR: OnceLock<RwLock<Option<Interceptor>>> = OnceLock::new();

fn slot() -> &'static RwLock<Option<Interceptor>> {
    INTERCEPTOR.get_or_init(|| RwLock::new(None))
}

/// Replace only at adapter startup/shutdown. A callback already executing keeps
/// its snapshot; no installation lock is held while any user callback runs.
pub fn set_interceptor(interceptor: Option<Interceptor>) -> Option<Interceptor> {
    std::mem::replace(
        &mut *slot().write().expect("packet bridge lock poisoned"),
        interceptor,
    )
}

fn snapshot() -> Option<Interceptor> {
    slot().read().expect("packet bridge lock poisoned").clone()
}

pub fn incoming(session: u64, packet: &mut Clientbound) -> bool {
    let mut event = PacketEvent::new(
        PacketContext {
            session,
            origin: TransferOrigin::Incoming,
            delivery: Delivery::Original,
        },
        packet,
    );
    if let Some(interceptor) = snapshot() {
        interceptor.incoming(&mut event);
    }
    event.is_cancelled()
}

pub fn outgoing(session: u64, packet: &mut Serverbound, silent: bool) -> bool {
    let mut event = PacketEvent::new(
        PacketContext {
            session,
            origin: TransferOrigin::Outgoing,
            delivery: if silent {
                Delivery::Silent
            } else {
                Delivery::Original
            },
        },
        packet,
    );
    if let Some(interceptor) = snapshot() {
        interceptor.outgoing(&mut event);
    }
    // NetworkUtils.sendPacketSilently invokes its three tracking services and
    // sends unconditionally; it does not inspect PacketEvent.isCancelled.
    !silent && event.is_cancelled()
}

pub fn cancel_local_swing() -> bool {
    snapshot().is_some_and(|interceptor| interceptor.cancel_local_swing())
}

/// MixinEntity's turn/absSnapRotationTo/setXRot hooks are process-wide, just as
/// in the original client mixin. NoPitchLimit.ServerSide only affects packets.
pub fn pitch_limit_removed() -> bool {
    snapshot().is_some_and(|interceptor| interceptor.bypass_pitch_limit())
}

pub fn pitch_limit(value: f32, min: f32, max: f32) -> f32 {
    if pitch_limit_removed() {
        value
    } else {
        value.clamp(min, max)
    }
}

/// Preserve the raw setXRot argument while enabled, including turns past 360.
/// Existing call sites retain their original finite/NaN checks.
pub fn set_x_rot(value: f32) -> f32 {
    if pitch_limit_removed() {
        value
    } else {
        (value % 360.0).clamp(-90.0, 90.0)
    }
}

thread_local! {
    static READER_SESSION: Cell<u64> = const { Cell::new(0) };
}

pub fn reader_session() -> u64 {
    READER_SESSION.with(Cell::get)
}

/// Cannot move to another thread: it restores that reader thread's previous id.
pub struct SessionScope {
    previous: u64,
    _thread: PhantomData<Rc<()>>,
}

pub fn session_scope(session: u64) -> SessionScope {
    SessionScope {
        previous: READER_SESSION.with(|current| current.replace(session)),
        _thread: PhantomData,
    }
}

impl Drop for SessionScope {
    fn drop(&mut self) {
        READER_SESSION.with(|current| current.set(self.previous));
    }
}

/// Ownership and connection generation travel with a delayed incoming packet.
/// Replaying calls the supplied real handler directly, never PacketInterceptor.
pub struct IncomingReplay {
    pub context: PacketContext,
    pub packet: Clientbound,
}

impl IncomingReplay {
    pub fn new(session: u64, packet: Clientbound) -> Self {
        Self {
            context: PacketContext {
                session,
                origin: TransferOrigin::Incoming,
                delivery: Delivery::IncomingReplay,
            },
            packet,
        }
    }

    pub fn apply<E>(
        self,
        current_session: u64,
        handler: impl FnOnce(Clientbound) -> Result<(), E>,
    ) -> Result<(), ReplayError<E>> {
        if self.context.session != current_session || current_session == 0 {
            return Err(ReplayError::StaleSession);
        }
        handler(self.packet).map_err(ReplayError::Handler)
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ReplayError<E> {
    StaleSession,
    Handler(E),
}
