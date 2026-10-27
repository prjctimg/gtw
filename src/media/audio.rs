use std::sync::Arc;
use std::time::Duration;

use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, OutputStreamBuilder, Sink};
use symphonia::core::audio::SampleBuffer;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::errors::Error as SymphoniaError;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

use crate::core::LoadError;
use crate::media::AudioData;

/// Decode a compressed audio file to interleaved f32 samples.
pub fn decode_audio(data: &[u8], title: impl Into<String>) -> Result<AudioData, LoadError> {
    let title = title.into();
    let source = std::io::Cursor::new(data.to_vec());
    let stream = MediaSourceStream::new(Box::new(source), Default::default());
    let mut hint = Hint::new();
    for extension in ["ogg", "oga", "opus", "mp3", "flac", "wav", "m4a"] {
        if title.to_lowercase().contains(extension) {
            hint.with_extension(extension);
            break;
        }
    }
    let probed = symphonia::default::get_probe()
        .format(
            &hint,
            stream,
            &FormatOptions {
                enable_gapless: true,
                ..Default::default()
            },
            &MetadataOptions::default(),
        )
        .map_err(|e| LoadError::parse(format!("audio container: {e}")))?;
    let mut format = probed.format;
    let track = format
        .default_track()
        .ok_or_else(|| LoadError::parse("audio file has no track"))?;
    let track_id = track.id;
    let mut decoder = symphonia::default::get_codecs()
        .make(&track.codec_params, &DecoderOptions::default())
        .map_err(|e| LoadError::parse(format!("audio codec: {e}")))?;

    let initial_capacity = track.codec_params.n_frames.unwrap_or(0).min(2_000_000) as usize;
    let mut samples: Vec<f32> = Vec::with_capacity(initial_capacity);
    let mut sample_rate = track.codec_params.sample_rate.unwrap_or(44_100);
    let mut channels = track
        .codec_params
        .channels
        .map(|c| c.count())
        .unwrap_or(2)
        .max(1) as u16;

    loop {
        let packet = match format.next_packet() {
            Ok(packet) => packet,
            Err(SymphoniaError::IoError(error))
                if error.kind() == std::io::ErrorKind::UnexpectedEof =>
            {
                break;
            }
            Err(SymphoniaError::ResetRequired) => break,
            Err(error) => return Err(LoadError::parse(format!("audio packet: {error}"))),
        };
        if packet.track_id() != track_id {
            continue;
        }
        let decoded = match decoder.decode(&packet) {
            Ok(decoded) => decoded,
            Err(SymphoniaError::DecodeError(_)) => continue,
            Err(error) => return Err(LoadError::parse(format!("audio decode: {error}"))),
        };
        let spec = *decoded.spec();
        sample_rate = spec.rate;
        channels = spec.channels.count().max(1) as u16;
        let mut buffer = SampleBuffer::<f32>::new(decoded.capacity() as u64, spec);
        buffer.copy_interleaved_ref(decoded);
        samples.extend_from_slice(buffer.samples());
        if samples.len() > 48_000 * 60 * 60 {
            break;
        }
    }
    if samples.is_empty() {
        return Err(LoadError::parse("audio file decoded to no samples"));
    }
    let frame_count = samples.len() as f64 / f64::from(channels.max(1));
    let duration = frame_count / f64::from(sample_rate.max(1));
    Ok(AudioData {
        samples,
        sample_rate,
        channels,
        duration,
        title,
    })
}

/// Thin playback surface over rodio. The output stream must stay alive for
/// the duration of playback, so it lives here next to the sink.
pub struct AudioEngine {
    /// Kept alive for the lifetime of the device; dropping it stops playback.
    #[allow(dead_code)]
    stream: OutputStream,
    sink: Sink,
    loaded_key: Option<String>,
    duration: f64,
    volume: f32,
    paused: bool,
}

impl AudioEngine {
    pub fn new() -> Result<Self, String> {
        let mut stream = OutputStreamBuilder::open_default_stream()
            .map_err(|e| format!("audio output unavailable: {e}"))?;
        stream.log_on_drop(false);
        let sink = Sink::connect_new(stream.mixer());
        sink.set_volume(0.75);
        Ok(Self {
            stream,
            sink,
            loaded_key: None,
            duration: 0.0,
            volume: 0.75,
            paused: false,
        })
    }

    pub fn play(&mut self, key: &str, data: &Arc<AudioData>) -> Result<(), String> {
        if self.loaded_key.as_deref() == Some(key) {
            self.sink.play();
            self.paused = false;
            return Ok(());
        }
        self.sink.stop();
        let source = SamplesBuffer::new(
            data.channels.max(1),
            data.sample_rate.max(1),
            data.samples.clone(),
        );
        self.sink.append(source);
        self.sink.set_volume(self.volume);
        self.sink.play();
        self.duration = data.duration;
        self.loaded_key = Some(key.to_string());
        self.paused = false;
        Ok(())
    }

    pub fn pause(&mut self) {
        self.sink.pause();
        self.paused = true;
    }

    pub fn stop(&mut self) {
        self.sink.stop();
        self.loaded_key = None;
        self.duration = 0.0;
        self.paused = false;
    }

    pub fn seek(&self, seconds: f64) {
        let clamped = seconds.clamp(0.0, self.duration.max(0.0));
        let _ = self.sink.try_seek(Duration::from_secs_f64(clamped));
    }

    pub fn set_volume(&mut self, volume: f32) {
        self.volume = volume.clamp(0.0, 1.0);
        self.sink.set_volume(self.volume);
    }

    pub fn volume(&self) -> f32 {
        self.volume
    }

    pub fn status(&self) -> (f64, bool) {
        (
            self.sink
                .get_pos()
                .as_secs_f64()
                .min(self.duration.max(0.0)),
            !self.paused && !self.sink.empty(),
        )
    }

    pub fn _keep_stream_alive(&self) -> &OutputStream {
        &self.stream
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_audio() {
        let error = decode_audio(b"not audio", "x.ogg").unwrap_err();
        assert!(error.message.contains("audio"));
    }

    #[test]
    fn decodes_pcm_wav() {
        let samples: Vec<i16> = (0..800)
            .map(|index| ((index as f32 * 0.2).sin() * 8000.0) as i16)
            .collect();
        let data_len = (samples.len() * 2) as u32;
        let mut wav = Vec::with_capacity(44 + data_len as usize);
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(36 + data_len).to_le_bytes());
        wav.extend_from_slice(b"WAVEfmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
        wav.extend_from_slice(&1u16.to_le_bytes()); // mono
        wav.extend_from_slice(&8_000u32.to_le_bytes());
        wav.extend_from_slice(&16_000u32.to_le_bytes());
        wav.extend_from_slice(&2u16.to_le_bytes());
        wav.extend_from_slice(&16u16.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_len.to_le_bytes());
        for sample in samples {
            wav.extend_from_slice(&sample.to_le_bytes());
        }
        let audio = decode_audio(&wav, "tone.wav").expect("decode wav");
        assert_eq!(audio.sample_rate, 8_000);
        assert_eq!(audio.channels, 1);
        assert_eq!(audio.samples.len(), 800);
        assert!((audio.duration - 0.1).abs() < 0.01);
    }
}
