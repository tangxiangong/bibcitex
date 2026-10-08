//! Native, cached TeX layout. No browser or external renderer is involved.
use bibcitex_service::{ChunkKind, ChunkRecord};
use gpui_kit::component::ActiveTheme;
use gpui_kit::{prelude::FluentBuilder as _, *};
use latex_rust::{MathFont, SvgOptions, latex_to_svg};
use sha2::{Digest, Sha256};
use std::{
    borrow::Cow,
    collections::HashMap,
    sync::{Mutex, OnceLock},
};

#[derive(Clone)]
struct Formula {
    data: Vec<u8>,
    width: f32,
    height: f32,
}
#[derive(Clone)]
enum Entry {
    Pending,
    Ready(Formula),
    Failed,
}
static CACHE: OnceLock<Mutex<HashMap<String, Entry>>> = OnceLock::new();
fn asset_key(source: &str) -> String {
    let digest: String = Sha256::digest(source.as_bytes())
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect();
    format!("math/{digest}.svg")
}
fn cache() -> &'static Mutex<HashMap<String, Entry>> {
    CACHE.get_or_init(Default::default)
}

pub fn asset(path: &str) -> Option<Cow<'static, [u8]>> {
    match cache().lock().ok()?.get(path)? {
        Entry::Ready(formula) => Some(Cow::Owned(formula.data.clone())),
        _ => None,
    }
}
fn dimension(svg: &str, name: &str) -> Option<f32> {
    svg.split_once(&format!("{name}=\""))?
        .1
        .split_once("pt\"")?
        .0
        .parse::<f32>()
        .ok()
        .map(|v| v * 4. / 3.)
}
fn request(source: &str, cx: &mut App) -> Option<Formula> {
    let key = asset_key(source);
    let mut entries = cache().lock().ok()?;
    match entries.get(&key) {
        Some(Entry::Ready(formula)) => return Some(formula.clone()),
        Some(_) => return None,
        None => {}
    }
    if source.len() > 8192 {
        return None;
    }
    // Keep memory bounded across many libraries. GPU assets are still keyed by content.
    if entries.len() >= 512 {
        entries.retain(|_, entry| matches!(entry, Entry::Pending));
    }
    entries.insert(key.clone(), Entry::Pending);
    drop(entries);
    let source = source.to_owned();
    let job = cx.background_executor().spawn(async move {
        let result = MathFont::stix_two_math()
            .and_then(|font| latex_to_svg(&source, &font, &SvgOptions::new()));
        let entry = match result {
            Ok(svg) => {
                let width = dimension(&svg, "width").unwrap_or(32.).clamp(1., 4096.);
                let height = dimension(&svg, "height").unwrap_or(20.).clamp(1., 4096.);
                Entry::Ready(Formula {
                    data: svg.into_bytes(),
                    width,
                    height,
                })
            }
            Err(_) => Entry::Failed,
        };
        if let Ok(mut entries) = cache().lock() {
            entries.insert(key, entry);
        }
    });
    cx.spawn(async move |cx| {
        job.await;
        cx.update(|cx| cx.refresh_windows());
    })
    .detach();
    None
}

pub fn rich(chunks: &[ChunkRecord], cx: &mut App) -> AnyElement {
    let mut line = div().flex().flex_wrap().items_center().gap_x_0().min_w_0();
    for chunk in chunks {
        if matches!(chunk.kind, ChunkKind::Math) {
            if let Some(formula) = request(&chunk.text, cx) {
                let key = asset_key(&chunk.text);
                line = line.child(
                    svg()
                        .path(key)
                        .text_color(cx.theme().foreground)
                        .w(px(formula.width))
                        .h(px(formula.height))
                        .flex_shrink_0(),
                );
            } else {
                line = line.child(format!("${}$", chunk.text));
            }
        } else {
            line = line.child(div().min_w_0().child(chunk.text.clone()));
        }
    }
    line.when(chunks.is_empty(), |line| line.child("—"))
        .into_any_element()
}
