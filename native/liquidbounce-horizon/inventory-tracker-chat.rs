// SPDX-License-Identifier: GPL-3.0-or-later
// Appended inside horizon-client-render::chat, beside its private pending list.
impl ChatScreen {
    pub fn liquidbounce_inventory_suggestions(
        &mut self,
        measure: &Measure,
        range: (usize, usize),
        suggestions: Vec<String>,
    ) {
        if range.0 > range.1
            || range.1 > self.input.value.len()
            || !self.input.value.is_char_boundary(range.0)
            || !self.input.value.is_char_boundary(range.1)
        {
            return;
        }
        self.request = None;
        self.awaiting = None;
        self.parse = None;
        self.usage.clear();
        self.usage_width = 0;
        self.usage_position = 0;
        self.input.suggestion = None;
        self.suggestions = None;
        self.pending = Some((
            range.0,
            suggestions
                .into_iter()
                .map(|text| Suggestion {
                    range,
                    text,
                    tooltip: None,
                    int_value: None,
                })
                .collect(),
        ));
        if self.allow_suggestions && self.auto_suggestions {
            self.show_suggestions(measure, false);
        }
    }
}
