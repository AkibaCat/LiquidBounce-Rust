// SPDX-License-Identifier: GPL-3.0-or-later
// This is included in native Font's module to use its real glyph atlas and cache.
include!("target-renderer-font-data.rs");
include!("target-renderer-font-registry.rs");
fn target_font_scale(font: &ab_glyph::FontVec) -> ab_glyph::PxScale {
    ab_glyph::PxScale::from(43. * font.height_unscaled() / font.units_per_em().unwrap_or(1000.))
}
impl Font {
    pub fn liquidbounce_target_metrics(&self) -> (f32, f32) {
        let (font, _) = target_font(false);
        let f = font.as_scaled(target_font_scale(&font));
        (
            f.ascent().ceil(),
            f.ascent().ceil() + (-f.descent()).ceil() + f.line_gap().ceil(),
        )
    }
    pub fn liquidbounce_target_glyph(&self, c: char, bold: bool) -> Glyph {
        let key = if bold {
            "liquidbounce:target_bold"
        } else {
            "liquidbounce:target"
        };
        let mut cache = self.cache.borrow_mut();
        let Cache { sets, region, .. } = &mut *cache;
        let index = if let Some(i) = sets.iter().position(|(id, _)| id == key) {
            i
        } else {
            sets.push((
                key.into(),
                FontSet {
                    providers: vec![],
                    glyphs: Default::default(),
                    non_fishy: Default::default(),
                    by_width: None,
                },
            ));
            sets.len() - 1
        };
        if let Some(g) = sets[index].1.glyphs.get(&c) {
            return *g;
        }
        let (mut font, mut embolden) = target_font(bold);
        let mut id = font.glyph_id(c);
        if id.0 == 0 {
            for cjk in [false, true] {
                if let Some(fallback) = target_fallback(bold, cjk) {
                    if fallback.glyph_id(c).0 != 0 {
                        font = fallback;
                        id = font.glyph_id(c);
                        embolden = false;
                        break;
                    }
                }
            }
        }
        let scaled = font.as_scaled(target_font_scale(&font));
        let bold_width = if embolden { 2 } else { 0 };
        let advance = scaled.h_advance(id) + bold_width as f32;
        let mut result = Glyph {
            advance,
            offset: 2.,
            ..Glyph::default()
        };
        if !self.metrics_only {
            if let Some(outline) = font.outline_glyph(
                id.with_scale_and_position(target_font_scale(&font), ab_glyph::point(0., 0.)),
            ) {
                let bounds = outline.px_bounds();
                let (w, h) = (bounds.width() as u32 + bold_width, bounds.height() as u32);
                if w > 0 && h > 0 {
                    let mut pixels = vec![0u8; (w * h * 4) as usize];
                    outline.draw(|x, y, a| {
                        for dx in 0..=bold_width {
                            let i = ((y * w + x + dx) * 4) as usize;
                            pixels[i..i + 3].fill(255);
                            pixels[i + 3] = pixels[i + 3].max((a * 255.).round() as u8);
                        }
                    });
                    if let Some((x, y)) = region.place(w, h, pixels, self.atlas, self.region) {
                        result.rect = Some(glyph_rect(x, y, w, h, self.atlas));
                        result.left = bounds.min.x;
                        result.top = bounds.min.y;
                        result.right = bounds.max.x + bold_width as f32;
                        result.bottom = bounds.max.y;
                    } else {
                        return self.missing;
                    }
                }
            }
        }
        sets[index].1.glyphs.insert(c, result);
        result
    }
}
