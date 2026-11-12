use crate::gif_utils::{GifMode, register_gif_edit_meme};

macro_rules! register_gif_speed {
    ($key:literal, $keyword:literal, $speed:expr) => {
        register_gif_edit_meme!(
            $key,
            &[$keyword],
            GifMode::Speed($speed),
            (2026, 11, 12),
            (2026, 11, 12),
        );
    };
}

register_gif_speed!("gif_speed_2", "加速2", 2);
register_gif_speed!("gif_speed_3", "加速3", 3);
register_gif_speed!("gif_speed_4", "加速4", 4);
register_gif_speed!("gif_speed_5", "加速5", 5);
register_gif_speed!("gif_speed_6", "加速6", 6);
register_gif_speed!("gif_speed_7", "加速7", 7);
register_gif_speed!("gif_speed_8", "加速8", 8);
register_gif_speed!("gif_speed_9", "加速9", 9);
register_gif_speed!("gif_speed_10", "加速10", 10);

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use skia_safe::{Codec, Data};

    use meme_generator_core::{
        error::Error,
        meme::{self, Meme},
    };
    use meme_generator_utils::{
        encoder::{GifEncoder, encode_png},
        image::ImageExt,
        tools::grid_pattern_image,
    };

    use crate::{gif_utils::plan_accelerated_frames, registry::MemeDeclaration};

    #[test]
    fn test_plan_keeps_all_frames_when_above_min_delay() {
        let durations = vec![200.0; 10];
        let plan = plan_accelerated_frames(&durations, 5);
        assert_eq!(plan.len(), 10);
        for (i, &(idx, dur)) in plan.iter().enumerate() {
            assert_eq!(idx, i);
            assert!((dur - 0.04).abs() < 1e-6);
        }
    }

    #[test]
    fn test_plan_resamples_when_below_min_delay() {
        // 30 frames * 50ms = 1500ms, 5x -> 300ms -> 15 frames * 20ms
        let durations = vec![50.0; 30];
        let plan = plan_accelerated_frames(&durations, 5);
        assert_eq!(plan.len(), 15);
        let total: f32 = plan.iter().map(|&(_, d)| d).sum();
        assert!((total - 0.30).abs() < 1e-5);
        assert!(plan.iter().all(|&(_, d)| d >= 0.02 - 1e-6));
        assert_eq!(plan[0].0, 0);
        assert_eq!(plan[1].0, 2);
        assert_eq!(plan[14].0, 28);
    }

    #[test]
    fn test_plan_coalesces_long_frames() {
        // Frame 0 is 1000ms, frames 1..10 are 20ms => total 1200ms, 2x => 600ms
        let mut durations = vec![1000.0];
        durations.extend(vec![20.0; 10]);
        let plan = plan_accelerated_frames(&durations, 2);
        assert!(plan.len() < 11);
        assert!(plan.len() >= 2);
        assert_eq!(plan[0].0, 0);
        assert!((plan[0].1 - 0.5).abs() < 0.03);
        let total: f32 = plan.iter().map(|&(_, d)| d).sum();
        assert!((total - 0.60).abs() < 1e-4);
    }

    #[test]
    fn test_plan_tiny_gif_keeps_at_least_two_frames() {
        let durations = vec![20.0, 20.0];
        let plan = plan_accelerated_frames(&durations, 10);
        assert_eq!(plan.len(), 2);
        assert!(plan.iter().all(|&(_, d)| d >= 0.02 - 1e-6));
    }

    #[test]
    fn test_registered_commands_and_generation() {
        let mut found: Vec<(u8, Box<dyn Meme>)> = Vec::new();
        for decl in inventory::iter::<MemeDeclaration> {
            if let Some(rest) = decl.name.strip_prefix("gif_speed_") {
                let speed: u8 = rest.parse().unwrap();
                found.push((speed, (decl.builder)()));
            }
        }
        found.sort_by_key(|(s, _)| *s);
        let speeds: Vec<u8> = found.iter().map(|(s, _)| *s).collect();
        assert_eq!(speeds, vec![2, 3, 4, 5, 6, 7, 8, 9, 10]);

        // Static PNG should return MemeFeedback
        let static_png = encode_png(grid_pattern_image().resize_exact((64, 64))).unwrap();
        let err = found[0]
            .1
            .generate(
                vec![meme::Image {
                    name: "test.png".to_string(),
                    data: static_png,
                }],
                vec![],
                HashMap::new(),
            )
            .unwrap_err();
        assert!(matches!(err, Error::MemeFeedback(_)));

        // Build a 20-frame 100ms GIF (total 2000ms) and test 2x..10x
        let img = grid_pattern_image().resize_exact((64, 64));
        let mut enc = GifEncoder::new();
        for i in 0..20 {
            enc.add_frame(img.rotate_crop((i * 18) as f32), 0.1).unwrap();
        }
        let sample_gif = enc.finish().unwrap();

        for (speed, meme) in &found {
            let info = meme.info();
            assert_eq!(info.keywords, vec![format!("加速{speed}")]);
            assert!(info.params.options.is_empty());
            assert_eq!(info.params.min_images, 1);
            assert_eq!(info.params.max_images, 1);
            assert_eq!(info.params.min_texts, 0);
            assert_eq!(info.params.max_texts, 0);

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
            assert!(count >= 2);
            let mut total_ms = 0;
            for i in 0..count {
                let d = codec.get_frame_info(i).unwrap().duration;
                assert!(d >= 20, "speed {speed} frame {i} duration {d} < 20ms");
                total_ms += d;
            }
            let expected_ms = 2000.0 / (*speed as f64);
            assert!(
                (total_ms as f64 - expected_ms).abs() <= 40.0,
                "speed {speed}: got {total_ms}ms, expected ~{expected_ms}ms"
            );

            let preview = meme.generate_preview(HashMap::new()).unwrap();
            let mut preview_codec = Codec::from_data(Data::new_copy(&preview)).unwrap();
            assert!(preview_codec.get_frame_count() >= 2);
        }
    }
}
