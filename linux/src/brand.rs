//! Scale the original wordmark to device pixels before handing it to the GPU.
use gpui_kit::{App, ImageCacheError, Img, RenderImage, Styled, Window, img, px};
use std::{cell::RefCell, collections::HashMap, sync::Arc};

thread_local! {
    static LOGOS: RefCell<HashMap<u32, Arc<RenderImage>>> = RefCell::new(HashMap::new());
}

pub fn logo(size: f32) -> Img {
    img(move |window: &mut Window, _: &mut App| {
        let pixels = (size * window.scale_factor()).round().max(1.) as u32;
        Some(LOGOS.with(|cache| {
            let mut cache = cache.borrow_mut();
            if let Some(image) = cache.get(&pixels) {
                return Ok(image.clone());
            }
            let source = image::load_from_memory_with_format(
                include_bytes!("../../public/favicon.png"),
                image::ImageFormat::Png,
            )
            .map_err(|error| ImageCacheError::Image(Arc::new(error)))?;
            // Direct GPU minification loses the wordmark's thin strokes at toolbar sizes.
            let mut bitmap = source
                .resize_exact(pixels, pixels, image::imageops::FilterType::Lanczos3)
                .into_rgba8();
            for pixel in bitmap.pixels_mut() {
                pixel.0.swap(0, 2); // GPUI RenderImage stores BGRA, just like its PNG loader.
            }
            let image = Arc::new(RenderImage::new(vec![image::Frame::new(bitmap)]));
            cache.insert(pixels, image.clone());
            Ok(image)
        }))
    })
    .size(px(size))
}
