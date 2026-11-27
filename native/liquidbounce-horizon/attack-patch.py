"""Actual attack and stopDestroyBlock phases against pinned Horizon sources."""


def apply(edit, replace_once):
    def native(text):
        addition = '''    /// Minecraft.startAttack: cooldown-read/write and HEAD miss cancellation.
    pub(super) fn native_client_attack_queries(&self, remove_cooldown: &mut bool, cancel_on_miss: &mut bool) {
        if self.native_window() {
            mod_api::dispatch_client_attack_queries(self, mod_api::mods(), remove_cooldown, cancel_on_miss);
        }
    }
    /// MultiPlayerGameMode.stopDestroyBlock HEAD; does not stop item use or clear keys.
    pub(super) fn native_client_abort_breaking(&self, canceled: &mut bool) {
        if self.native_window() {
            mod_api::dispatch_client_abort_breaking(self, mod_api::mods(), canceled);
        }
    }

'''
        return replace_once(text, '    /// Integer Options.fov value before Camera.calculateFov / getNearPlane.\n', addition + '    /// Integer Options.fov value before Camera.calculateFov / getNearPlane.\n')

    def first_person(text):
        text = replace_once(text, '    fn hit(&mut self) -> Hit;\n', '''    fn hit(&mut self) -> Hit;
    /// Local client module decisions; pure/server hand worlds preserve vanilla.
    fn client_attack_queries(&self) -> (bool, bool) { (false, false) }
    fn client_abort_breaking(&self) -> bool { false }
''')
        text = replace_once(text, '''    pub fn start_attack(&mut self, w: &mut impl HandWorld) -> bool {
        if self.miss_time > 0 || w.hands_busy() {
''', '''    pub fn start_attack(&mut self, w: &mut impl HandWorld) -> bool {
        let (remove_cooldown, cancel_on_miss) = w.client_attack_queries();
        // LiquidBounce's HEAD injection returns true before the vanilla guards
        // and before a spear attack, without swinging or resetting strength.
        if cancel_on_miss && matches!(w.hit(), Hit::Miss) { return true; }
        if (!remove_cooldown && self.miss_time > 0) || w.hands_busy() {
''')
        text = replace_once(text, '''        if miss {
            if !w.creative() {
                self.miss_time = 10;
''', '''        if miss {
            if !w.creative() && !remove_cooldown {
                self.miss_time = 10;
''')
        return replace_once(text, '''    pub fn stop(&mut self, w: &mut impl HandWorld) {
        if self.active {
''', '''    pub fn stop(&mut self, w: &mut impl HandWorld) {
        if w.client_abort_breaking() { return; }
        if self.active {
''')

    def hand(text):
        text = replace_once(text, '''    pub(super) fn stop_mining(&mut self) {
        if self.clicks.destroying.active {
''', '''    pub(super) fn stop_mining(&mut self) {
        let mut retain_break = false;
        self.native_client_abort_breaking(&mut retain_break);
        if self.clicks.destroying.active && !retain_break {
''')
        text = replace_once(text, '''        self.request_remote_mining_stop();
        self.mining = None;
''', '''        self.request_remote_mining_stop();
        if !retain_break { self.mining = None; }
''')
        return replace_once(text, '''impl HandWorld for GameHands<'_, '_> {
    fn hit(&mut self) -> Hit {
''', '''impl HandWorld for GameHands<'_, '_> {
    fn client_attack_queries(&self) -> (bool, bool) {
        let (mut remove, mut cancel) = (false, false);
        self.game.native_client_attack_queries(&mut remove, &mut cancel);
        (remove, cancel)
    }
    fn client_abort_breaking(&self) -> bool {
        let mut canceled = false;
        self.game.native_client_abort_breaking(&mut canceled);
        canceled
    }
    fn hit(&mut self) -> Hit {
''')

    def remote(text):
        text = replace_once(text, '''    fn remote_start_attack(&mut self) -> bool {
        if self.clicks.miss_time > 0 || self.hands_busy() {
''', '''    fn remote_start_attack(&mut self) -> bool {
        let (mut remove_cooldown, mut cancel_on_miss) = (false, false);
        self.native_client_attack_queries(&mut remove_cooldown, &mut cancel_on_miss);
        if cancel_on_miss && !self.entity_targeted() && self.pick().is_none() { return true; }
        if (!remove_cooldown && self.clicks.miss_time > 0) || self.hands_busy() {
''')
        text = replace_once(text, '''            if !self.creative {
                self.clicks.miss_time = 10;
''', '''            if !self.creative && !remove_cooldown {
                self.clicks.miss_time = 10;
''')
        # These callers also keep a separate crack-rendering target. A canceled
        # stop must preserve it, while vanilla's nonactive stop still clears it.
        for indent in ('        ', '            '):
            original = indent + 'self.stop_remote_mining();\n' + indent + 'self.mining = None;'
            replacement = indent + '''let mut retain_break = false;
''' + indent + '''self.native_client_abort_breaking(&mut retain_break);
''' + indent + '''self.stop_remote_mining();
''' + indent + '''if !retain_break { self.mining = None; }'''
            text = text.replace(original, replacement)
        return replace_once(text, '''    pub(super) fn stop_remote_mining(&mut self) -> bool {
        let kept = self.remote.as_mut().and_then(|r| {
''', '''    pub(super) fn stop_remote_mining(&mut self) -> bool {
        let mut canceled = false;
        self.native_client_abort_breaking(&mut canceled);
        if canceled { return false; }
        let kept = self.remote.as_mut().and_then(|r| {
''')

    edit('crates/horizon-client/src/game/native_hooks.rs', native)
    edit('crates/horizon-client/src/first_person.rs', first_person)
    edit('crates/horizon-client/src/game/hand.rs', hand)
    edit('crates/horizon-client/src/game/remote.rs', remote)
    return ['client_attack_queries', 'client_abort_breaking'], []
