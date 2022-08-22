use std::time::Duration;

use image::{imageops, DynamicImage};
use reqwest::Url;
use teloxide::types::{InlineKeyboardButton, InlineKeyboardButtonKind, InlineKeyboardMarkup};

#[derive(Clone, Copy)]
pub struct CollageOptions {
    pub image_count: (u32, u32),
    pub image_size: (u32, u32),
    pub gap: u32,
}

pub fn image_collage<I: IntoIterator<Item = DynamicImage>>(
    images: I,
    options: CollageOptions,
) -> DynamicImage {
    let size = (
        options.image_count.0 * options.image_size.0 + (options.image_count.0 - 1) * options.gap,
        options.image_count.1 * options.image_size.1 + (options.image_count.1 - 1) * options.gap,
    );
    let mut base = DynamicImage::new_rgb8(size.0, size.1);

    for (i, image) in images.into_iter().enumerate() {
        let col = i % options.image_count.0 as usize;
        let row = i / options.image_count.0 as usize;
        let x = col * (options.image_size.0 + options.gap) as usize;
        let y = row * (options.image_size.1 + options.gap) as usize;
        imageops::overlay(&mut base, &image, x as _, y as _);
    }

    base
}

pub fn format_duration(duration: Duration) -> String {
    let duration = duration.as_secs();
    let hours = (duration / 3600) % 60;
    let minutes = (duration / 60) % 60;
    let seconds = duration % 60;

    if hours > 0 {
        format!("{hours}h {minutes}m {seconds}s")
    } else if minutes > 0 {
        format!("{minutes}m {seconds}s")
    } else {
        format!("{seconds}s")
    }
}

pub fn donate_markup<S: AsRef<str>>(name: S, url: S) -> InlineKeyboardMarkup {
    InlineKeyboardMarkup::new([[InlineKeyboardButton::new(
        format!("Donate to {}", name.as_ref()),
        InlineKeyboardButtonKind::Url(Url::parse(url.as_ref()).unwrap()),
    )]])
}
