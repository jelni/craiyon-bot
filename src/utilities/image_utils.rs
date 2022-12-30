use image::{imageops, DynamicImage};

pub fn collage(
    images: Vec<DynamicImage>,
    image_size: (u32, u32),
    image_count_x: u32,
    gap: u32,
) -> DynamicImage {
    #[allow(clippy::pedantic)] // multiple lossy numeric conversions
    let image_count_y = (images.len() as f32 / image_count_x as f32).ceil() as u32;
    let mut base = DynamicImage::new_rgb8(
        image_count_x * image_size.0 + (image_count_x - 1) * gap,
        image_count_y * image_size.1 + (image_count_y - 1) * gap,
    );

    for (i, image) in images.into_iter().enumerate() {
        let col = i % image_count_x as usize;
        let row = i / image_count_x as usize;
        let x = col * (image_size.0 + gap) as usize;
        let y = row * (image_size.1 + gap) as usize;
        imageops::overlay(&mut base, &image, x as _, y as _);
    }

    base
}
