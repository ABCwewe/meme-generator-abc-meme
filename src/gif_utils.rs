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

pub(crate) const MIN_FRAME_MS: f64 = 20.0;
const DEFAULT_FRAME_MS: f64 = 100.0;

#[derive(Clone, Copy)]
pub(crate) enum GifMode {
    Speed(u8),
    Slow(u8),
    Reverse,
}

pub(crate) struct GifEditMeme {
    pub key: &'static str,
    pub keywords: &'static [&'static str],
    pub mode: GifMode,
    pub date_created: (i32, u32, u32),
    pub date_modified: (i32, u32, u32),
}

impl Meme for GifEditMeme {
    fn key(&self) -> String {
        self.key.to_string()
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
            keywords: self.keywords.iter().map(|s| (*s).to_string()).collect(),
            shortcuts: Vec::new(),
            tags: HashSet::new(),
            date_created: local_date(self.date_created.0, self.date_created.1, self.date_created.2),
            date_modified: local_date(
                self.date_modified.0,
                self.date_modified.1,
                self.date_modified.2,
            ),
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
        transform_gif(&images[0].data, self.mode)
    }

    fn generate_preview(&self, _options: HashMap<String, OptionValue>) -> Result<Vec<u8>, Error> {
        let preview_gif = make_preview_gif()?;
        transform_gif(&preview_gif, self.mode)
    }
}

/// 根据原始每帧时长（毫秒）和加速倍率，计算输出帧序列 `(原帧索引, 输出时长秒)`。
///
/// 当加速后的单帧间隔低于 GIF 播放器安全下限（20ms，50fps）时，按时间轴重采样抽帧，
/// 避免浏览器/社交软件将过小帧间隔回退为 100ms 导致反向变慢。
pub(crate) fn plan_accelerated_frames(durations_ms: &[f64], speed: u8) -> Vec<(usize, f32)> {
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

pub(crate) fn plan_slowed_frames(durations_ms: &[f64], factor: u8) -> Vec<(usize, f32)> {
    let factor_f = factor as f64;
    durations_ms
        .iter()
        .enumerate()
        .map(|(i, &d)| (i, ((d * factor_f).clamp(MIN_FRAME_MS, 655_000.0) / 1000.0) as f32))
        .collect()
}

pub(crate) fn plan_reversed_frames(durations_ms: &[f64]) -> Vec<(usize, f32)> {
    durations_ms
        .iter()
        .enumerate()
        .rev()
        .map(|(i, &d)| (i, (d.max(MIN_FRAME_MS) / 1000.0) as f32))
        .collect()
}

pub(crate) fn transform_gif(data: &[u8], mode: GifMode) -> Result<Vec<u8>, Error> {
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

    let plan = match mode {
        GifMode::Speed(speed) => plan_accelerated_frames(&durations_ms, speed),
        GifMode::Slow(factor) => plan_slowed_frames(&durations_ms, factor),
        GifMode::Reverse => plan_reversed_frames(&durations_ms),
    };

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

macro_rules! register_gif_edit_meme {
    ($key:literal, $keywords:expr, $mode:expr, $date_created:expr, $date_modified:expr $(,)?) => {
        inventory::submit! {
            $crate::registry::MemeDeclaration {
                name: $key,
                builder: || -> Box<dyn meme_generator_core::meme::Meme> {
                    Box::new($crate::gif_utils::GifEditMeme {
                        key: $key,
                        keywords: $keywords,
                        mode: $mode,
                        date_created: $date_created,
                        date_modified: $date_modified,
                    })
                },
            }
        }
    };
}

pub(crate) use register_gif_edit_meme;

