use std::collections::{HashMap, HashSet};

use skia_safe::{Codec, Data};

use meme_generator_core::{
    error::Error,
    meme::{self, Meme, MemeInfo, MemeParams, OptionValue},
};
use meme_generator_utils::{
    decoder::CodecExt,
    encoder::GifEncoder,
    image::ImageExt,
    tools::{grid_pattern_image, local_date},
};

const MIN_FRAME_MS: f64 = 20.0;
const DEFAULT_FRAME_MS: f64 = 100.0;

struct GifSpeedMeme {
    speed: u8,
}

impl Meme for GifSpeedMeme {
    fn key(&self) -> String {
        format!("gif_speed_{}", self.speed)
    }

    fn info(&self) -> MemeInfo {
        MemeInfo {
            key: self.key(),
            params: MemeParams {
                min_images: 1,
                max_images: 1,
                min_texts: 0,
                max_texts: 0,
                default_texts: Vec::new(),
                options: Vec::new(),
            },
            keywords: vec![format!("加速{}", self.speed)],
            shortcuts: Vec::new(),
            tags: HashSet::new(),
            date_created: local_date(2026, 11, 12),
            date_modified: local_date(2026, 11, 12),
        }
    }

    fn generate(
        &self,
        images: Vec<meme::Image>,
        texts: Vec<String>,
        _options: HashMap<String, OptionValue>,
    ) -> Result<Vec<u8>, Error> {
        if images.len() != 1 {
            return Err(Error::ImageNumberMismatch(1, 1, images.len() as u8));
        }
        if !texts.is_empty() {
            return Err(Error::TextNumberMismatch(0, 0, texts.len() as u8));
        }
        accelerate_gif(&images[0].data, self.speed)
    }

    fn generate_preview(&self, _options: HashMap<String, OptionValue>) -> Result<Vec<u8>, Error> {
        let preview_gif = make_preview_gif()?;
        accelerate_gif(&preview_gif, self.speed)
    }
}

/// 根据原始每帧时长（毫秒）和加速倍率，计算输出帧序列 `(原帧索引, 输出时长秒)`。
///
/// 当加速后的单帧间隔低于 GIF 播放器安全下限（20ms，50fps）时，按时间轴重采样抽帧，
/// 避免浏览器/社交软件将过小帧间隔回退为 100ms 导致反向变慢。
fn plan_accelerated_frames(durations_ms: &[f64], speed: u8) -> Vec<(usize, f32)> {
    let speed_f = speed as f64;
    if durations_ms.iter().all(|&d| d / speed_f >= MIN_FRAME_MS) {
        return durations_ms
            .iter()
            .enumerate()
            .map(|(i, &d)| (i, (d / speed_f / 1000.0) as f32))
            .collect();
    }

    let total_orig_ms: f64 = durations_ms.iter().sum();
    let total_target_ms = total_orig_ms / speed_f;
    let max_samples = ((total_target_ms + 1e-6) / MIN_FRAME_MS).floor() as usize;
    let sample_count = max_samples.max(2);
    let sample_duration_sec = ((total_target_ms / sample_count as f64) / 1000.0)
        .max(MIN_FRAME_MS / 1000.0) as f32;

    let mut ends = Vec::with_capacity(durations_ms.len());
    let mut acc = 0.0;
    for &d in durations_ms {
        acc += d;
        ends.push(acc);
    }

    let step_orig_ms = total_orig_ms / sample_count as f64;
    let mut out: Vec<(usize, f32)> = Vec::with_capacity(sample_count);
    let mut frame_idx = 0;

    for k in 0..sample_count {
        let t = k as f64 * step_orig_ms;
        while frame_idx + 1 < durations_ms.len() && ends[frame_idx] <= t + 1e-6 {
            frame_idx += 1;
        }
        if let Some((last_idx, last_dur)) = out.last_mut()
            && *last_idx == frame_idx
        {
            *last_dur += sample_duration_sec;
        } else {
            out.push((frame_idx, sample_duration_sec));
        }
    }

    if out.len() == 1 && durations_ms.len() >= 2 {
        out.push((durations_ms.len() - 1, sample_duration_sec));
    }

    out
}

fn accelerate_gif(data: &[u8], speed: u8) -> Result<Vec<u8>, Error> {
    let sk_data = Data::new_copy(data);
    let mut codec =
        Codec::from_data(sk_data).ok_or(Error::ImageDecodeError("Skia decode error".to_string()))?;

    if !codec.is_multi_frame() {
        return Err(Error::MemeFeedback("请发送动图（GIF）".to_string()));
    }

    let count = codec.get_frame_count();
    let mut durations_ms = Vec::with_capacity(count);
    for i in 0..count {
        let frame_info = codec
            .get_frame_info(i)
            .ok_or(Error::ImageDecodeError("Skia decode error".to_string()))?;
        let d = if frame_info.duration > 0 {
            frame_info.duration as f64
        } else {
            DEFAULT_FRAME_MS
        };
        durations_ms.push(d);
    }

    let plan = plan_accelerated_frames(&durations_ms, speed);
    let mut encoder = GifEncoder::new();
    for (frame_index, duration) in plan {
        let frame = codec.get_frame(frame_index)?;
        encoder.add_frame(frame, duration)?;
    }
    encoder.finish()
}

fn make_preview_gif() -> Result<Vec<u8>, Error> {
    let img = grid_pattern_image().resize_exact((120, 120));
    let mut encoder = GifEncoder::new();
    for i in 0..8 {
        let rotated = img.rotate_crop((i * 45) as f32);
        encoder.add_frame(rotated, 0.2)?;
    }
    encoder.finish()
}

macro_rules! register_gif_speed {
    ($key:literal, $speed:expr) => {
        inventory::submit! {
            $crate::registry::MemeDeclaration {
                name: $key,
                builder: || -> Box<dyn meme_generator_core::meme::Meme> {
                    Box::new(GifSpeedMeme { speed: $speed })
                },
            }
        }
    };
}

register_gif_speed!("gif_speed_2", 2);
register_gif_speed!("gif_speed_3", 3);
register_gif_speed!("gif_speed_4", 4);
register_gif_speed!("gif_speed_5", 5);
register_gif_speed!("gif_speed_6", 6);
register_gif_speed!("gif_speed_7", 7);
register_gif_speed!("gif_speed_8", 8);
register_gif_speed!("gif_speed_9", 9);
register_gif_speed!("gif_speed_10", 10);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registry::MemeDeclaration;
    use meme_generator_utils::encoder::encode_png;

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

        // Build a 20-frame 100ms GIF (total 2000ms) and test 2x and 10x
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
