//! A cancelled setScreen keeps the previous Screen's menu object. Horizon's
//! normal fields remain the active player menu between native UI consumers.
use super::*;

#[derive(Clone, Default)]
pub(super) struct RemoteView {
    pub(super) menu: Option<(i32, String)>,
    pub(super) title: Option<horizon_client::text::Text>,
    pub(super) result: Option<ItemStack>,
}

pub(super) struct ContainerView {
    serial: u64,
    screen: Screen,
    title: Option<(Option<String>, horizon_client::text::Text)>,
    remote: RemoteView,
    carried: Option<ItemStack>,
    craft_grid: Vec<Option<ItemStack>>,
    craft_size: usize,
    craft_table: Option<[i32; 3]>,
    container: Option<(i32, i32, i32)>,
    open_container_slots: Option<((i32, i32, i32), usize)>,
    station: Option<stations::Station>,
    mount: mount_menu::MountMenu,
    mod_screen: Option<native_hooks::OpenNativeScreen>,
    // A remote menu's client container is independent of the next menu,
    // even when Horizon reuses the same synthetic block-entity key.
    entities: Vec<((i32, i32, i32), Option<BlockEntity>)>,
}

impl ContainerView {
    fn exchange(&mut self, game: &mut Game<'_>) {
        use std::mem::swap;
        swap(&mut self.carried, &mut game.carried);
        swap(&mut self.craft_grid, &mut game.craft_grid);
        swap(&mut self.craft_size, &mut game.craft_size);
        swap(&mut self.craft_table, &mut game.craft_table);
        swap(&mut self.container, &mut game.container);
        swap(&mut self.open_container_slots, &mut game.open_container_slots);
        swap(&mut self.station, &mut game.station);
        swap(&mut self.mount, &mut game.mount_menu);
        swap(&mut self.mod_screen, &mut game.mod_screen);
        for (key, entity) in &mut self.entities {
            let next = entity.take();
            *entity = game.world.block_entities.remove(key);
            if let Some(next) = next { game.world.block_entities.insert(*key, next); }
        }
    }
}

impl Game<'_> {
    pub(super) fn client_gui_closer_finish_screen(&mut self, accepted: bool) -> bool {
        // Discard only after the real lifecycle observer changes the epoch.
        // A native screen provider may still fail before assigning a Screen.
        accepted
    }
    pub(super) fn client_gui_closer_abandon_menu(&mut self) {
        if let Some(mut view) = self.client_gui_closer_view.take() { view.exchange(self); }
    }
    fn client_gui_closer_kept(&self) -> Option<&ContainerView> {
        self.client_gui_closer_view.as_ref().filter(|view| {
            view.serial == self.client_inventory_screen_serial.get() && view.screen == self.screen
        })
    }

    /// Before constructing player.containerMenu. Move owned menu objects;
    /// never copy Remote, its state id, socket, packet state or player inventory.
    pub(super) fn client_gui_closer_prepare_menu(&mut self) {
        if self.client_gui_closer_in_view || self.client_gui_closer_other.is_some() || self.client_gui_closer_kept().is_some()
            || !self.native_window() || self.lan_acting.is_some() || self.lan_acting_uuid.is_some()
            || !matches!(self.screen, Screen::Inventory | Screen::Creative) { return; }
        self.client_gui_closer_view = None;
        let title = self.client_container_menu_text(self.screen);
        let remote = self.remote.as_ref().map_or_else(RemoteView::default, |r| RemoteView {
            menu: r.menu.clone(), title: r.menu_title.clone(), result: r.craft_result.clone(),
        });
        let entities = if self.remote.is_some() {
            self.container_parts().into_iter().map(|(key, _)| {
                (key, self.world.block_entities.get(&key).cloned())
            }).collect()
        } else { Vec::new() };
        self.client_gui_closer_view = Some(ContainerView {
            serial: self.client_inventory_screen_serial.get(), screen: self.screen, title, remote,
            carried: self.carried.clone(), craft_grid: self.craft_grid.clone(),
            craft_size: self.craft_size, craft_table: self.craft_table, container: self.container,
            open_container_slots: self.open_container_slots, station: self.station.take(),
            mount: std::mem::take(&mut self.mount_menu), mod_screen: self.mod_screen.take(), entities,
        });
    }

    pub(super) fn client_gui_closer_display_title(&self) -> Option<(Option<String>, String)> {
        let plain = |title: &horizon_client::text::Text| {
            horizon_client::scoreboard::comp_text(&title.component()).resolve_legacy().plain()
        };
        if self.client_gui_closer_in_view {
            return self.client_gui_closer_display_title.as_ref().map(|(menu, title)| (menu.clone(), plain(title)));
        }
        self.client_gui_closer_kept()?.title.as_ref().map(|(menu, title)| (menu.clone(), plain(title)))
    }

    /// UI scopes expose the previous Screen's menu; all other code sees the
    /// newly constructed player menu. Nested native input stays in this scope.
    pub(super) fn with_gui_closer_view<R>(&mut self, run: impl FnOnce(&mut Self) -> R) -> R {
        if self.client_gui_closer_in_view { return run(self); }
        if self.client_gui_closer_kept().is_none() { self.client_gui_closer_view = None; return run(self); }
        let mut other = self.client_gui_closer_view.take().expect("checked view");
        self.client_gui_closer_display_remote = Some(other.remote.clone());
        self.client_gui_closer_display_title = other.title.clone();
        other.exchange(self);
        self.client_gui_closer_other = Some(other);
        self.client_gui_closer_in_view = true;
        let result = run(self);
        let mut other = self.client_gui_closer_other.take().expect("active menu context");
        other.exchange(self);
        self.client_gui_closer_in_view = false;
        self.client_gui_closer_display_remote = None;
        self.client_gui_closer_display_title = None;
        if other.serial == self.client_inventory_screen_serial.get() && other.screen == self.screen {
            self.client_gui_closer_view = Some(other);
        }
        result
    }

    /// Network and inventory Host operations may be called reentrantly by a
    /// native screen or a screen-render listener. They always see the active menu.
    pub(super) fn with_gui_closer_active<R>(&mut self, run: impl FnOnce(&mut Self) -> R) -> R {
        if !self.client_gui_closer_in_view { return run(self); }
        let mut other = self.client_gui_closer_other.take().expect("active menu context");
        other.exchange(self);
        self.client_gui_closer_other = Some(other);
        let remote = self.client_gui_closer_display_remote.take();
        let title = self.client_gui_closer_display_title.take();
        self.client_gui_closer_in_view = false;
        let result = run(self);
        let mut other = self.client_gui_closer_other.take().expect("screen menu context");
        other.exchange(self);
        self.client_gui_closer_other = Some(other);
        self.client_gui_closer_in_view = true;
        self.client_gui_closer_display_remote = remote;
        self.client_gui_closer_display_title = title;
        result
    }

    pub(super) fn client_gui_closer_stale_input(&self) -> bool {
        self.client_gui_closer_in_view && self.remote.is_some()
            && self.client_gui_closer_display_remote.as_ref().map_or(0, |v| v.menu.as_ref().map_or(0, |m| m.0))
                != self.remote.as_ref().and_then(|r| r.menu.as_ref()).map_or(0, |m| m.0)
    }
}
