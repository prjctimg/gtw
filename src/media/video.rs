use std::sync::Arc;

use ::image::RgbaImage;
use ::image::imageops::FilterType;
use zvidlib::io::MemorySource;
use zvidlib::{
    CancellationToken, Codec, CodecProfile, ColorRange, ExactFrameReader, FrameIndex,
    HardwarePreference, Limits, PixelFormat, TrackKind, VideoDecoderConfig, WebmDemuxer,
    WebmDemuxerOptions, native_vp8_video_decoder_factory, native_vp9_video_decoder_factory,
};

use crate::core::LoadError;

const MAX_PREVIEW_FRAMES: usize = 180;
const MAX_PREVIEW_WIDTH: u32 = 640;
const MAX_PREVIEW_BYTES: usize = 96 * 1024 * 1024;

#[derive(Clone, Debug)]
pub struct VideoClip {
    pub frames: Vec<Arc<RgbaImage>>,
    pub fps: f64,
    pub duration: f64,
}

impl VideoClip {
    pub fn total(&self) -> u32 {
        self.frames.len() as u32
    }

    pub fn frame(&self, index: u32) -> Option<Arc<RgbaImage>> {
        self.frames.get(index as usize).cloned()
    }

    pub fn index_at(&self, position: f64) -> u32 {
        ((position * self.fps).floor().max(0.0) as u32).min(self.total().saturating_sub(1))
    }
}

/// Decode the first few seconds of a WebM (VP8/VP9) video to a bounded set of
/// RGBA preview frames. Pure Rust all the way down: no ffmpeg, no subprocess.
pub fn decode_video(data: &[u8]) -> Result<VideoClip, LoadError> {
    let source = MemorySource::new(data.to_vec());
    let demuxer = pollster::block_on(WebmDemuxer::open(&source, WebmDemuxerOptions::default()))
        .map_err(|e| LoadError::parse(format!("video container: {e}")))?;
    let track = demuxer
        .tracks
        .iter()
        .find(|track| track.kind == TrackKind::Video)
        .ok_or_else(|| LoadError::parse("video has no visual track"))?;
    let dimensions = track
        .dimensions
        .ok_or_else(|| LoadError::parse("video track has no dimensions"))?;
    if dimensions.width == 0 || dimensions.height == 0 {
        return Err(LoadError::parse("video track has invalid dimensions"));
    }
    let limits = Limits {
        max_width: dimensions.width.max(1),
        max_height: dimensions.height.max(1),
        ..Limits::default()
    };
    let mut samples = pollster::block_on(track.to_encoded_video_samples(&source, &limits))
        .map_err(|e| LoadError::parse(format!("video samples: {e}")))?;
    if samples.is_empty() {
        return Err(LoadError::parse("video contains no frames"));
    }
    let source_frames = track.presentation_order.len().max(1);
    let bytes_per_frame = (dimensions.width as usize)
        .saturating_mul(dimensions.height as usize)
        .saturating_mul(4)
        .max(1);
    let by_memory = (MAX_PREVIEW_BYTES / bytes_per_frame).max(1);
    let frame_limit = source_frames
        .min(MAX_PREVIEW_FRAMES)
        .min(by_memory.max(24))
        .max(1);
    samples.retain(|sample| sample.presentation_index.0 < frame_limit as u64);
    if samples.is_empty() {
        return Err(LoadError::parse("video frame index is empty"));
    }

    let (factory, profile): (Box<dyn zvidlib::VideoDecoderFactory>, CodecProfile) =
        match track.codec {
            Codec::Vp9 => (
                Box::new(native_vp9_video_decoder_factory()),
                CodecProfile::Vp9Profile0,
            ),
            Codec::Vp8 => (
                Box::new(native_vp8_video_decoder_factory()),
                CodecProfile::Vp8,
            ),
            other => {
                return Err(LoadError::parse(format!(
                    "unsupported video codec {other:?}; use a VP8/VP9 WebM preview"
                )));
            }
        };
    let configuration = VideoDecoderConfig {
        codec: track.codec,
        profile,
        coded_dimensions: dimensions,
        output_format: PixelFormat::Rgba8,
        color_range: ColorRange::Limited,
        hardware: HardwarePreference::Avoid,
        configuration: track.decoder_config.clone(),
    };
    let mut reader = ExactFrameReader::new(&*factory, configuration, samples, limits)
        .map_err(|e| LoadError::parse(format!("video decoder: {e}")))?;
    let cancel = CancellationToken::new();
    let target_width = dimensions.width.min(MAX_PREVIEW_WIDTH);
    let target_height = ((u64::from(target_width) * u64::from(dimensions.height))
        / u64::from(dimensions.width.max(1)))
    .max(1) as u32;
    let mut frames = Vec::new();
    for index in 0..frame_limit {
        let frame = reader
            .get(FrameIndex(index as u64), &cancel)
            .map_err(|e| LoadError::parse(format!("video decode frame {index}: {e}")))?;
        let rgba = frame_to_rgba(&frame)?;
        let rgba = if rgba.width() > MAX_PREVIEW_WIDTH {
            ::image::imageops::resize(&rgba, target_width, target_height, FilterType::Triangle)
        } else {
            rgba
        };
        frames.push(Arc::new(rgba));
    }
    let duration = if track.timescale > 0 {
        track.duration as f64 / f64::from(track.timescale)
    } else {
        frames.len() as f64 / 24.0
    };
    let count = frames.len() as f64;
    let fps = if duration > 0.1 {
        (count / duration).clamp(1.0, 60.0)
    } else {
        24.0
    };
    let preview_duration = count / fps.max(0.001);
    Ok(VideoClip {
        frames,
        fps,
        duration: preview_duration,
    })
}

fn frame_to_rgba(frame: &zvidlib::VideoFrame) -> Result<RgbaImage, LoadError> {
    if frame.pixel_format != PixelFormat::Rgba8 {
        return Err(LoadError::parse(format!(
            "video decoder returned {:?}, expected RGBA",
            frame.pixel_format
        )));
    }
    let width = frame.dimensions.width;
    let height = frame.dimensions.height;
    let plane = frame
        .planes
        .first()
        .ok_or_else(|| LoadError::parse("video frame has no pixel plane"))?;
    let row_bytes = width as usize * 4;
    if plane.stride < row_bytes || plane.data.len() < plane.stride * height as usize {
        return Err(LoadError::parse("video frame plane is truncated"));
    }
    let mut data = Vec::with_capacity(row_bytes * height as usize);
    for y in 0..height as usize {
        let start = y * plane.stride;
        data.extend_from_slice(&plane.data[start..start + row_bytes]);
    }
    RgbaImage::from_raw(width, height, data)
        .ok_or_else(|| LoadError::parse("video frame buffer is invalid"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_video_input() {
        let error = decode_video(b"not a webm file").unwrap_err();
        assert!(error.message.contains("video"));
    }

    #[test]
    fn decodes_vp8_webm_preview() {
        // Synthetic 64x48, 12-frame VP8 WebM generated by zvidlib's encoder.
        // Keeping it in the tree makes video preview coverage network-free.
        let clip = decode_video(include_bytes!("../../tests/fixtures/tiny-vp8.webm"))
            .expect("decode vp8 fixture");
        assert_eq!(clip.total(), 12);
        assert!(
            clip.frames
                .iter()
                .all(|frame| frame.width() == 64 && frame.height() == 48)
        );
        assert_eq!(clip.frames[0].as_raw().len(), 64 * 48 * 4);
        assert!(clip.duration > 0.0);
    }

    /// Manual check for a real VP9 preview without making the test suite hit
    /// the network. Download a WebM and point `GTW_WEBM_FIXTURE` at it.
    #[test]
    fn decodes_webm_fixture_from_env_when_set() {
        let Ok(path) = std::env::var("GTW_WEBM_FIXTURE") else {
            return;
        };
        let data = std::fs::read(&path).expect("read GTW_WEBM_FIXTURE");
        let clip = decode_video(&data).expect("decode GTW_WEBM_FIXTURE");
        assert!(!clip.frames.is_empty());
        assert!(clip.frames[0].width() > 0);
    }
}
