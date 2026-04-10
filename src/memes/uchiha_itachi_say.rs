use skia_safe::{Color, IRect};

use meme_generator_core::error::Error;
use meme_generator_utils::{
    builder::InputImage,
    encoder::encode_png,
    image::ImageExt,
    tools::{load_image, local_date, new_paint},
};

use crate::{custom_fn::draw_text_vertical_auto_font_size, options::NoOptions, register_meme};

fn uchiha_itachi_say(_: Vec<InputImage>, texts: Vec<String>, _: NoOptions) -> Result<Vec<u8>, Error> {
    let mut text = &texts[0];
    let frame = load_image("uchiha_itachi_say/background.png")?;
    let mut surface = frame.to_surface();
    let canvas = surface.canvas();
    let textnb = [String::from("鸣潮牛逼"),String::from("原神牛逼")];
    let nb_name = ["牛逼","牛福","nb"];
    let wuwa_name= ["wuwa","wutheringwaves","鸣潮","明潮","明朝","鸣朝","mingchao","minchao","mingcao","mincao","mc"];
    let gs_name = ["genshin","genshinimpact","gs","yuanshen","yuansen","yuanseng","yuansheng","yuangshen","yuangsen","yuangseng","yuangsheng","原神","ys","o神"];
    let wuwanb: Vec<String> = wuwa_name
        .iter()
        .flat_map(|&p| {
            nb_name.iter().map(move |&s| format!("{}{}", p, s))
        })
        .collect();
    let gsnb: Vec<String> = gs_name
        .iter()
        .flat_map(|&p| {
            nb_name.iter().map(move |&s| format!("{}{}", p, s))
        })
        .collect();



    if wuwanb.contains(&keep_chinese_and_lowercase(text)) {
        text = &textnb[1];
    }
    else if gsnb.contains(&keep_chinese_and_lowercase(text)) {
        text = &textnb[0];
    }

    // canvas.draw_text_area_auto_font_size(
    //     IRect::from_ltrb(270, 181, 308, 333),
    //     text,
    //     20.0,
    //     80.0,
    //     text_params!(
    //         font_families = &["Noto Sans SC"],
    //         text_align = TextAlign::Center,
    //         paint = new_paint(Color::from_rgb(0, 0, 0)),
    //     ),
    // )?;

    draw_text_vertical_auto_font_size(
        canvas,
        IRect::from_ltrb(270, 181, 308, 333),
        text,
        20.0,
        80.0,
        &["Noto Sans CJK SC"],
        &new_paint(Color::from_rgb(0, 0, 0)),
    );
    encode_png(surface.image_snapshot())
}

/// 判断字符是否为中文字符（基本区 + 扩展A区）
fn is_chinese(c: char) -> bool {
    (c >= '\u{4e00}' && c <= '\u{9fff}')   // CJK 统一表意文字
        || (c >= '\u{3400}' && c <= '\u{4dbf}') // 扩展A区
}

/// 保留中文字符和英文小写字母，丢弃其他所有字符，并将大写字母转为小写
fn keep_chinese_and_lowercase(s: &str) -> String {
    s.chars()
        .filter_map(|c| {
            if c.is_ascii_alphabetic() {
                // 英文字母 -> 转小写后保留
                Some(c.to_ascii_lowercase())
            } else if is_chinese(c) {
                // 中文字符 -> 直接保留
                Some(c)
            } else {
                // 数字、标点、空格等全部丢弃
                None
            }
        })
        .collect()
}

register_meme!(
    "uchiha_itachi_say",
    uchiha_itachi_say,
    min_images = 0,
    max_images = 0,
    min_texts = 1,
    max_texts = 1,
    default_texts = &["牛逼"],
    keywords = &["yzbys"],
    date_created = local_date(2026, 4, 10),
    date_modified = local_date(2026, 4, 10),
);