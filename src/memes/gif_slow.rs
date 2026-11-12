use crate::gif_utils::{GifMode, register_gif_edit_meme};

macro_rules! register_gif_slow {
    ($key:literal, $keyword:literal, $factor:expr) => {
        register_gif_edit_meme!(
            $key,
            &[$keyword],
            GifMode::Slow($factor),
            (2026, 11, 13),
            (2026, 11, 13),
        );
    };
}

register_gif_slow!("gif_slow_2", "减速2", 2);
register_gif_slow!("gif_slow_3", "减速3", 3);
register_gif_slow!("gif_slow_4", "减速4", 4);
register_gif_slow!("gif_slow_5", "减速5", 5);
register_gif_slow!("gif_slow_6", "减速6", 6);
register_gif_slow!("gif_slow_7", "减速7", 7);
register_gif_slow!("gif_slow_8", "减速8", 8);
register_gif_slow!("gif_slow_9", "减速9", 9);
register_gif_slow!("gif_slow_10", "减速10", 10);

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use skia_safe::{Codec, Data};

    use meme_generator_core::meme::{self, Meme};
    use meme_generator_utils::{
        encoder::GifEncoder,
        image::ImageExt,
        tools::grid_pattern_image,
    };

    use crate::{gif_utils::plan_slowed_frames, registry::MemeDeclaration};

    #[test]
    fn test_plan_slowed_frames() {
        let durations = vec![50.0, 100.0, 10.0];
        let plan = plan_slowed_frames(&durations, 3);
        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0].0, 0);
        assert!((plan[0].1 - 0.15).abs() < 1e-6);
        assert_eq!(plan[1].0, 1);
        assert!((plan[1].1 - 0.30).abs() < 1e-6);
        assert_eq!(plan[2].0, 2);
        assert!((plan[2].1 - 0.03).abs() < 1e-6);
    }

    #[test]
    fn test_registered_slow_commands_and_generation() {
        let mut found: Vec<(u8, Box<dyn Meme>)> = Vec::new();
        for decl in inventory::iter::<MemeDeclaration> {
            if let Some(rest) = decl.name.strip_prefix("gif_slow_") {
                let factor: u8 = rest.parse().unwrap();
                found.push((factor, (decl.builder)()));
            }
        }
        found.sort_by_key(|(f, _)| *f);
        let factors: Vec<u8> = found.iter().map(|(f, _)| *f).collect();
        assert_eq!(factors, vec![2, 3, 4, 5, 6, 7, 8, 9, 10]);

        // Build a 10-frame 50ms GIF (total 500ms)
        let img = grid_pattern_image().resize_exact((64, 64));
        let mut enc = GifEncoder::new();
        for i in 0..10 {
            enc.add_frame(img.rotate_crop((i * 36) as f32), 0.05)
                .unwrap();
        }
        let sample_gif = enc.finish().unwrap();

        for (factor, meme) in &found {
            let info = meme.info();
            assert_eq!(info.keywords, vec![format!("减速{factor}")]);
            assert!(info.params.options.is_empty());
            assert_eq!(info.params.min_images, 1);
            assert_eq!(info.params.max_images, 1);

            let out = meme
                .generate(
                    vec![meme::Image {
                        name: "anim.gif".to_string(),
                        data: sample_gif.clone(),
                    }],
                    vec![],
                    HashMap::new(),
                )
                .unwrap();

            let mut codec = Codec::from_data(Data::new_copy(&out)).unwrap();
            let count = codec.get_frame_count();
            assert_eq!(count, 10);
            let mut total_ms = 0;
            for i in 0..count {
                total_ms += codec.get_frame_info(i).unwrap().duration;
            }
            let expected_ms = 500 * (*factor as i32);
            assert!(
                (total_ms - expected_ms).abs() <= 20,
                "factor {factor}: got {total_ms}ms, expected {expected_ms}ms"
            );

            let preview = meme.generate_preview(HashMap::new()).unwrap();
            let mut preview_codec = Codec::from_data(Data::new_copy(&preview)).unwrap();
            assert_eq!(preview_codec.get_frame_count(), 8);
        }
    }
}
