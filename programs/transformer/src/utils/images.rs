use image::{imageops, open, DynamicImage};

fn h_concat(mut base: DynamicImage, imgs: &[DynamicImage]) -> DynamicImage {
    for img in imgs {
        imageops::overlay(&mut base, img, 0, 0);
    }
    base
}

pub fn overlay_images() {
    let base = open("https://assets.stickpng.com/images/61d183263a856e0004c6334a.png").unwrap();

    h_concat(
        base,
        &[open("https://assets.stickpng.com/images/580b57fcd9996e24bc43c516.png").unwrap()],
    )
    .save("random.png")
    .unwrap();
}
