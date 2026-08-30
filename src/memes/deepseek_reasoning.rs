use skia_safe::{Color, IRect, textlayout::TextAlign};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::encode_png,
    image::ImageExt,
    text::Text2Image,
    text_params,
    tools::{load_image, local_date, new_paint},
};

use crate::{options::NoOptions, register_meme};

fn deepseek_reasoning(
    _: Vec<InputImage>,
    texts: Vec<String>,
    _: NoOptions,
) -> Result<Vec<u8>, Error> {
    let text = texts[0].clone();
    let frame = load_image("deepseek_reasoning/background.png")?;
    let mut surface = frame.to_surface();
    let canvas = surface.canvas();

    // 字号上限与背景图中“已思考（用时 0 秒）”的字号匹配
    let (max_font_size, min_font_size) = (40.0, 24.0);
    let rect = IRect::from_ltrb(161, 282, 618, 384);
    let area_width = (rect.right() - rect.left()) as f32;
    let area_height = (rect.bottom() - rect.top()) as f32;
    let params = text_params!(
        font_families = &["Microsoft YaHei"],
        text_align = TextAlign::Left,
        paint = new_paint(Color::from_rgb(0, 0, 0)),
    );

    let mut font_size = max_font_size;
    let mut text2image;
    loop {
        text2image = Text2Image::from_text(&text, font_size, params.clone());
        text2image.layout(area_width);
        if text2image.height() <= area_height {
            break;
        }
        if font_size <= min_font_size {
            return Err(Error::TextOverLength(text));
        }
        font_size -= 1.0;
    }
    text2image.draw_on_canvas(canvas, (rect.left() as f32, rect.top() as f32));
    encode_png(surface.image_snapshot())
}

register_meme!(
    "deepseek_reasoning",
    deepseek_reasoning,
    min_images = 0,
    max_images = 0,
    min_texts = 1,
    max_texts = 1,
    default_texts = &["让我想想..."],
    keywords = &["deepseek思考", "大肥鱼思考"],
    date_created = local_date(2026, 8, 30),
    date_modified = local_date(2026, 8, 30),
);
