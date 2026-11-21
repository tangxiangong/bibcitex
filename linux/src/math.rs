//! Cached, local typesetting for bibliography fields containing TeX.
use base64::{Engine as _, engine::general_purpose::STANDARD};
use latex_rust::{
    Color, Dim, MathFont, MathStyle, SvgOptions, layout_with_em_size_pt, parse,
    render_svg_with_metrics,
};
use nsh_core::{element::blocking, prelude::*};
use std::{
    collections::VecDeque,
    sync::{Mutex, OnceLock},
};

/// A formula's text, typeset artwork and layout metrics in logical points.
#[derive(Clone, Debug, PartialEq, JsObject)]
pub struct FormulaDto {
    /// Original TeX, including when the renderer cannot typeset it.
    pub source: String,
    /// A self-contained SVG URL that can be passed directly to an Image.
    pub data_url: Option<String>,
    /// The complete vector artwork as SVG markup.
    pub svg: Option<String>,
    /// Width of the artwork in logical points.
    pub width: f32,
    /// Height of the artwork in logical points.
    pub height: f32,
    /// Distance from the top of the artwork to the text baseline.
    pub baseline: f32,
}

impl FormulaDto {
    fn fallback(tex: &str) -> Self {
        Self {
            source: tex.to_owned(),
            data_url: None,
            svg: None,
            width: 0.,
            height: 0.,
            baseline: 0.,
        }
    }
    fn bytes(&self) -> usize {
        self.source.len()
            + self.data_url.as_ref().map_or(0, String::len)
            + self.svg.as_ref().map_or(0, String::len)
    }
}

#[derive(Default)]
struct FormulaCache {
    entries: VecDeque<(u32, bool, FormulaDto)>,
    bytes: usize,
}
impl FormulaCache {
    fn get(&mut self, tex: &str, font_size: f32, dark: bool) -> Option<FormulaDto> {
        let index = self.entries.iter().position(|(bits, tone, item)| {
            *bits == font_size.to_bits() && *tone == dark && item.source == tex
        })?;
        let entry = self.entries.remove(index)?;
        let formula = entry.2.clone();
        self.entries.push_back(entry);
        Some(formula)
    }
    fn insert(&mut self, font_size: f32, dark: bool, formula: FormulaDto) {
        const MAX_BYTES: usize = 16 * 1024 * 1024;
        let bytes = formula.bytes();
        if bytes > MAX_BYTES {
            return;
        }
        while self.entries.len() >= 512 || self.bytes + bytes > MAX_BYTES {
            if let Some((_, _, evicted)) = self.entries.pop_front() {
                self.bytes -= evicted.bytes();
            }
        }
        self.bytes += bytes;
        self.entries.push_back((font_size.to_bits(), dark, formula));
    }
}

fn typeset(tex: &str, font_size: f32, dark: bool) -> Option<FormulaDto> {
    static FONT: OnceLock<Option<MathFont>> = OnceLock::new();
    let font = FONT
        .get_or_init(|| MathFont::stix_two_math().ok())
        .as_ref()?;
    let color = if dark {
        Color::rgb(255, 255, 255)
    } else {
        Color::rgb(0, 0, 0)
    };
    let options = SvgOptions {
        font_size_pt: Dim::from_ieee32_bits(font_size.to_bits()),
        color,
        display: false,
    };
    let ast = parse(tex).ok()?;
    let tree = layout_with_em_size_pt(&ast, font, MathStyle::Text, &options.font_size_pt).ok()?;
    let image = render_svg_with_metrics(&tree, font, &options).ok()?;
    let data = STANDARD.encode(image.svg.as_bytes());
    Some(FormulaDto {
        source: tex.into(),
        data_url: Some(format!("data:image/svg+xml;base64,{data}")),
        svg: Some(image.svg),
        width: f32::from_bits(image.width_pt.to_ieee32_bits()),
        height: f32::from_bits(image.height_pt.to_ieee32_bits()),
        baseline: f32::from_bits(image.baseline_pt.to_ieee32_bits()),
    })
}

fn cached_formula(tex: String, font_size: f32, dark: bool) -> FormulaDto {
    static CACHE: OnceLock<Mutex<FormulaCache>> = OnceLock::new();
    if tex.is_empty() || tex.len() > 8192 || !font_size.is_finite() || font_size <= 0. {
        return FormulaDto::fallback(&tex);
    }
    // All calls run on blocking workers. Holding this lock through rendering also
    // coalesces concurrent requests by multiple windows for the same formula.
    let mut cache = CACHE
        .get_or_init(Default::default)
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    if let Some(formula) = cache.get(&tex, font_size, dark) {
        return formula;
    }
    let formula = typeset(&tex, font_size, dark).unwrap_or_else(|| FormulaDto::fallback(&tex));
    cache.insert(font_size, dark, formula.clone());
    formula
}

/// Renders inline TeX with the bundled STIX Two Math face.
#[module]
pub struct MathRender;

#[module]
impl MathRender {
    /// Typesets a formula at the given point size and appearance. Unsupported TeX retains its source.
    pub async fn formula(tex: String, font_size: f32, dark: bool) -> Result<FormulaDto, String> {
        blocking(move || cached_formula(tex, font_size, dark))
            .await
            .map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn formula_preserves_baseline_size_appearance_and_original_tex() {
        let tex = r"\frac{1}{2}";
        let normal = cached_formula(tex.into(), 12., false);
        assert_eq!(normal.source, tex);
        assert!(normal.width > 0. && normal.baseline > 0. && normal.baseline < normal.height);
        let data = normal
            .data_url
            .as_ref()
            .unwrap()
            .strip_prefix("data:image/svg+xml;base64,")
            .unwrap();
        assert_eq!(
            STANDARD.decode(data).unwrap(),
            normal.svg.as_ref().unwrap().as_bytes()
        );
        let large = cached_formula(tex.into(), 24., false);
        assert!(large.height > normal.height);
        assert_ne!(cached_formula(tex.into(), 12., true).svg, normal.svg);
        assert!(cached_formula(tex.into(), 0., false).svg.is_none());
        let mut cache = FormulaCache::default();
        cache.insert(12., false, normal.clone());
        assert_eq!(cache.get(tex, 12., false).unwrap(), normal);
        assert!(cache.get(tex, 12., true).is_none());
    }
}
