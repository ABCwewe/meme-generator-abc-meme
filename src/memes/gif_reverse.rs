use crate::gif_utils::{GifMode, register_gif_edit_meme};

register_gif_edit_meme!(
    "gif_reverse",
    &["倒带", "倒放"],
    GifMode::Reverse,
    (2026, 11, 13),
    (2026, 11, 13),
);

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use skia_safe::{Color, Codec, Data};

    use meme_generator_core::meme::{self, Meme};
    use meme_generator_utils::{
        decoder::CodecExt,
        encoder::GifEncoder,
        tools::new_surface,
    };

    use crate::{gif_utils::plan_reversed_frames, registry::MemeDeclaration};

    #[test]
    fn test_plan_reversed_frames() {
        let durations = vec![40.0, 80.0, 120.0];
        let plan = plan_reversed_frames(&durations);
        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0].0, 2);
        assert!((plan[0].1 - 0.12).abs() < 1e-6);
        assert_eq!(plan[1].0, 1);
        assert!((plan[1].1 - 0.08).abs() < 1e-6);
        assert_eq!(plan[2].0, 0);
        assert!((plan[2].1 - 0.04).abs() < 1e-6);
    }

    #[test]
    fn test_registered_reverse_command_and_generation() {
        let decl = inventory::iter::<MemeDeclaration>
            .into_iter()
            .find(|d| d.name == "gif_reverse")
            .expect("gif_reverse should be registered");
        let meme: Box<dyn Meme> = (decl.builder)();

        let info = meme.info();
        assert_eq!(info.keywords, vec!["倒带", "倒放"]);
        assert!(info.params.options.is_empty());
        assert_eq!(info.params.min_images, 1);
        assert_eq!(info.params.max_images, 1);

        // Build a 4-frame GIF with distinct colors and durations (gifski repeats the penultimate delay on the final frame)
        let colors = [Color::RED, Color::GREEN, Color::BLUE, Color::YELLOW];
        let durs = [0.06f32, 0.10, 0.14, 0.14];
        let mut enc = GifEncoder::new();
        for (&color, &dur) in colors.iter().zip(durs.iter()) {
            let mut surface = new_surface((32, 32));
            surface.canvas().clear(color);
            enc.add_frame(surface.image_snapshot(), dur).unwrap();
        }
        let sample_gif = enc.finish().unwrap();

        let out = meme
            .generate(
                vec![meme::Image {
                    name: "rgby.gif".to_string(),
                    data: sample_gif,
                }],
                vec![],
                HashMap::new(),
            )
            .unwrap();

        let mut codec = Codec::from_data(Data::new_copy(&out)).unwrap();
        assert_eq!(codec.get_frame_count(), 4);

        let expected_durs = [140, 140, 100, 100];
        let expected_colors = [Color::YELLOW, Color::BLUE, Color::GREEN, Color::RED];
        for i in 0..4 {
            let d = codec.get_frame_info(i).unwrap().duration;
            assert!(
                (d - expected_durs[i]).abs() <= 10,
                "frame {i}: got {d}ms, expected ~{}ms",
                expected_durs[i]
            );
            let frame = codec.get_frame(i).unwrap();
            let pix = frame.peek_pixels().unwrap();
            let c = pix.get_color((16, 16));
            let exp = expected_colors[i];
            assert!(
                (c.r() as i16 - exp.r() as i16).abs() <= 5
                    && (c.g() as i16 - exp.g() as i16).abs() <= 5
                    && (c.b() as i16 - exp.b() as i16).abs() <= 5,
                "frame {i} color mismatch: got {:?}, expected {:?}",
                c,
                exp
            );
        }

        let preview = meme.generate_preview(HashMap::new()).unwrap();
        let mut preview_codec = Codec::from_data(Data::new_copy(&preview)).unwrap();
        assert_eq!(preview_codec.get_frame_count(), 8);
    }
}
