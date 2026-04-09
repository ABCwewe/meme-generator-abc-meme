use skia_safe::IRect;

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    canvas::CanvasExt,
    encoder::encode_png,
    image::ImageExt,
    tools::{load_image, local_date},
};

use crate::{options::NoOptions, register_meme};

fn mutsumi_think(_: Vec<InputImage>, texts: Vec<String>, _: NoOptions) -> Result<Vec<u8>, Error> {
    let text = &texts[0];
    let frame = load_image("mutsumi_think/background.png")?;
    let mut surface = frame.to_surface();
    let canvas = surface.canvas();
    canvas.draw_text_area_auto_font_size(
        IRect::from_ltrb(53, 49, 190, 127),
        text,
        20.0,
        50.0,
        text_params!(
            font_families = &["FZShaoEr-M11S"],
            text_align = TextAlign::Center,
            paint = new_paint(Color::from_rgb(0, 0, 0)),
        ),
    )?;
    encode_png(surface.image_snapshot())
}

register_meme!(
    "mutsumi_think",
    mutsumi_think,
    min_images = 0,
    max_images = 0,
    min_texts = 1,
    max_texts = 1,
    default_texts = &["好女孩..."],
    keywords = &["木头想", "睦想"],
    date_created = local_date(2026, 4, 10),
    date_modified = local_date(2026, 4, 10),
);