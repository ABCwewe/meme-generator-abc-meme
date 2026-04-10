use skia_safe::{Canvas, Font, FontMgr, FontStyle, IRect, Paint, Point,};

pub fn draw_text_vertical_auto_font_size(
    canvas: &Canvas,
    rect: IRect,
    text: &str,
    min_font_size: f32,
    max_font_size: f32,
    font_families: &[&str],
    paint: &Paint,
) {  // <-- 改为返回 ()
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return;
    }

    let rect_w = (rect.right - rect.left) as f32;
    let rect_h = (rect.bottom - rect.top) as f32;

    let font_mgr = FontMgr::new();
    let typeface = font_families
        .iter()
        .find_map(|name| font_mgr.match_family_style(name, FontStyle::default()))
        .unwrap_or_else(|| {
            font_mgr
                .match_family_style("", FontStyle::default())
                .expect("failed to load default typeface")
        });

    let mut font_size = max_font_size;
    let step = 1.0_f32;

    let layout = loop {
        if font_size < min_font_size {
            font_size = min_font_size;
        }

        let rows = (rect_h / font_size).floor() as usize;
        let cols = (rect_w / font_size).floor() as usize;

        if rows > 0 && cols > 0 && chars.len() <= rows * cols {
            let font = Font::new(&typeface, font_size);
            let mut positions: Vec<(char, Point)> = Vec::with_capacity(chars.len());

            for (i, &ch) in chars.iter().enumerate() {
                let col = i / rows;
                let row = i % rows;
                let cell_x = rect.right as f32 - (col as f32 + 1.0) * font_size;
                let cell_y = rect.top as f32 + row as f32 * font_size;
                let s = ch.to_string();
                let (advance, _) = font.measure_str(&s, Some(paint));
                let char_x = cell_x + (font_size - advance) / 2.0;
                let char_y = cell_y + font_size;
                positions.push((ch, Point::new(char_x, char_y)));
            }

            break (font_size, positions);
        }

        if font_size <= min_font_size {
            let font = Font::new(&typeface, font_size);
            let rows = (rect_h / font_size).floor().max(1.0) as usize;
            let mut positions = Vec::new();

            for (i, &ch) in chars.iter().enumerate() {
                let col = i / rows;
                let row = i % rows;
                let cell_x = rect.right as f32 - (col as f32 + 1.0) * font_size;
                let cell_y = rect.top as f32 + row as f32 * font_size;
                let s = ch.to_string();
                let (advance, _) = font.measure_str(&s, Some(paint));
                let char_x = cell_x + (font_size - advance) / 2.0;
                positions.push((ch, Point::new(char_x, cell_y + font_size)));
            }

            break (font_size, positions);
        }

        font_size -= step;
    };

    let (chosen_size, positions) = layout;
    let font = Font::new(&typeface, chosen_size);

    for (ch, point) in positions {
        canvas.draw_str(ch.to_string(), point, &font, paint);
    }
}