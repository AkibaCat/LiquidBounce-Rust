// SPDX-License-Identifier: GPL-3.0-or-later
// Appended inside game::recipe_book to use its actual ClientBook/RecipeDisplayEntry.
impl Game<'_> {
    pub(super) fn client_auto_crafter_snapshot(&mut self, targets: &str) -> Option<String> {
        if !self.client_auto_tool_player() {
            return None;
        }
        let (kind, book_type) = match self.container_kind() {
            Some(BlockEntity::Furnace(f)) => match f.kind {
                FurnaceKind::Furnace => (2, BookType::Furnace),
                FurnaceKind::BlastFurnace => (3, BookType::BlastFurnace),
                FurnaceKind::Smoker => (4, BookType::Smoker),
            },
            None if self.container.is_none() && self.station.is_none() && !self.mount_active() => {
                if self.craft_size == 3 {
                    (1, BookType::Crafting)
                } else {
                    (0, BookType::Crafting)
                }
            }
            _ => return None,
        };
        // inventoryMenu persists when its GUI closes; opened container menus do not.
        let serial = if kind == 0 {
            0
        } else {
            self.client_inventory_screen_serial.get()
        };
        let open = self.book_open(book_type);
        let targets: Vec<String> = serde_json::from_str(targets).ok()?;
        let targets = targets
            .iter()
            .map(|id| ItemStack::new(id.as_str(), 1))
            .collect::<Vec<_>>();
        self.ensure_book();
        self.sync_client_book();
        let mut stock = Stacked::default();
        let mut result = None;
        self.menu_session(|menu, env, _| {
            stock = menu.stacked_contents(env);
            result = menu.slot(if kind >= 2 { 2 } else { 0 }).cloned();
            None
        });
        let book = self.recipe_ui.book.as_ref()?;
        let (gw, gh) = self.book_grid();
        let mut recipes = Vec::new();
        for collection in self
            .recipe_ui
            .client
            .collections(book, book_type, Tab::Search)
        {
            for index in collection.entries {
                let entry = &book.entries[index];
                let requirements = match book.server_requirements.get(&entry.display_id) {
                    Some(Some(v)) => v.iter().collect::<Vec<_>>(),
                    Some(None) => vec![],
                    None => book
                        .placements
                        .get(&entry.id)
                        .map_or_else(|| entry.requirements(), |p| p.ingredients.iter().collect()),
                };
                let uses = targets
                    .iter()
                    .filter(|stack| requirements.iter().any(|r| r.test(stack)))
                    .map(|s| s.item.to_string())
                    .collect::<Vec<_>>();
                recipes.push(serde_json::json!({"display":entry.display_id,"outputs":entry.results().iter().map(|s|s.item.to_string()).collect::<Vec<_>>(),"uses":uses,"craftable":(kind>=2||entry.fits(gw,gh))&&book.can_craft(entry,&stock)}));
            }
        }
        let capacity = result.as_ref().map_or(0, |stack| {
            let maximum = self.stack_max_size(stack);
            self.inventory
                .iter()
                .take(36)
                .map(|slot| match slot {
                    None => maximum,
                    Some(other) if stack.same_item_same_components(other) => {
                        maximum.saturating_sub(other.count)
                    }
                    _ => 0,
                })
                .sum::<u32>()
        });
        Some(serde_json::json!({"menu":[self.client_inventory_container()as u64,serial],"kind":kind,"open":open,"recipes":recipes,"result":result.as_ref().map_or(0,|s|s.count),"capacity":capacity}).to_string())
    }
    pub(super) fn client_auto_crafter_place(&mut self, display: i32, all: bool) {
        if !self.client_auto_tool_player() {
            return;
        }
        self.ensure_book();
        let Some(index) = self
            .recipe_ui
            .book
            .as_ref()
            .and_then(|b| b.entries.iter().position(|e| e.display_id == display))
        else {
            return;
        };
        let book = match self.container_kind() {
            Some(BlockEntity::Furnace(f)) => match f.kind {
                FurnaceKind::Furnace => BookType::Furnace,
                FurnaceKind::BlastFurnace => BookType::BlastFurnace,
                FurnaceKind::Smoker => BookType::Smoker,
            },
            _ => BookType::Crafting,
        };
        self.place_recipe(book, index, all);
    }
    pub(super) fn client_auto_crafter_result(&mut self, throw: bool) {
        if !self.client_auto_tool_player() {
            return;
        }
        let slot = if matches!(self.container_kind(), Some(BlockEntity::Furnace(_))) {
            2
        } else {
            0
        };
        self.native_client_inventory_click(
            self.client_inventory_container(),
            slot,
            if throw { 1 } else { 0 },
            if throw { 4 } else { 1 },
        );
    }
}
