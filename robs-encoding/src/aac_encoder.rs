use robs_core::traits::*;
use robs_core::*;
use anyhow::Result;
use async_trait::async_trait;
use std::any::Any;
use std::io::Write;
use std::process::ChildStdin;
use ffmpeg_sidecar::child::FfmpegChild;
use ffmpeg_sidecar::command::FfmpegCommand;
use ffmpeg_sidecar::event::FfmpegEvent;

pub struct FfmpegAacEncoder {
    id: EncoderId,
    name: String,
    bitrate: u32,
    sample_rate: u32,
    channels: u32,
    caps: EncoderCapabilities,
    stdin: Option<ChildStdin>,
    child: Option<FfmpegChild>,
    frame_count: i64,
    initialized: bool,
    input_info: Option<AudioInfo>,
}

impl FfmpegAacEncoder {
    pub fn new() -> Self {
        Self {
            id: EncoderId(ObjectId::new()),
            name: "FFmpeg AAC".to_string(),
            bitrate: 128,
            sample_rate: 48000,
            channels: 2,
            caps: EncoderCapabilities {
                max_width: 0, max_height: 0, max_fps: 0,
                supports_hardware: false,
                supported_pixel_formats: vec![],
            },
            stdin: None, child: None, frame_count: 0, initialized: false,
            input_info: None,
        }
    }

    pub fn is_available() -> bool {
        std::process::Command::new("ffmpeg")
            .args(["-codecs"])
            .output()
            .map(|o| {
                let combined = format!(
                    "{}{}",
                    String::from_utf8_lossy(&o.stdout),
                    String::from_utf8_lossy(&o.stderr)
                );
                combined.contains("AAC")
            })
            .unwrap_or(false)
    }
}

impl Default for FfmpegAacEncoder { fn default() -> Self { Self::new() } }

#[async_trait]
impl Encoder for FfmpegAacEncoder {
    fn id(&self) -> EncoderId { self.id }
    fn name(&self) -> &str { &self.name }
    fn codec_name(&self) -> &str { "aac" }
    fn media_type(&self) -> MediaType { MediaType::Audio }
    fn get_input_info(&self) -> Option<MediaInfo> {
        self.input_info.as_ref().map(|a| MediaInfo::Audio(AudioEncodeInfo {
            audio: a.clone(), encoder_name: "aac".to_string(),
            codec_params: serde_json::json!({}),
        }))
    }
    fn get_output_info(&self) -> Option<MediaInfo> { None }
    fn get_caps(&self) -> EncoderCapabilities { self.caps.clone() }

    fn get_presets(&self) -> Vec<EncoderPreset> {
        vec![
            EncoderPreset { id: 0, name: "64kbps".into(), description: "Low quality, voice only".into(), quality_level: 1, speed_level: 5 },
            EncoderPreset { id: 1, name: "96kbps".into(), description: "Low quality".into(), quality_level: 2, speed_level: 5 },
            EncoderPreset { id: 2, name: "128kbps".into(), description: "Standard quality".into(), quality_level: 4, speed_level: 5 },
            EncoderPreset { id: 3, name: "160kbps".into(), description: "Good quality".into(), quality_level: 5, speed_level: 5 },
            EncoderPreset { id: 4, name: "192kbps".into(), description: "High quality".into(), quality_level: 6, speed_level: 5 },
            EncoderPreset { id: 5, name: "256kbps".into(), description: "Very high quality".into(), quality_level: 7, speed_level: 5 },
            EncoderPreset { id: 6, name: "320kbps".into(), description: "Maximum quality".into(), quality_level: 8, speed_level: 5 },
        ]
    }
    fn get_current_preset(&self) -> EncoderPreset {
        self.get_presets().into_iter().find(|p| p.name == format!("{}kbps", self.bitrate))
            .unwrap_or_else(|| EncoderPreset { id: 2, name: "128kbps".into(), description: "Standard".into(), quality_level: 4, speed_level: 5 })
    }
    fn set_preset(&mut self, preset: EncoderPreset) -> Result<()> {
        let bitrate: u32 = preset.name.trim_end_matches("kbps").parse().unwrap_or(128);
        self.bitrate = bitrate;
        Ok(())
    }

    fn parameters_definition(&self) -> Vec<PropertyDef> {
        vec![
            PropertyDef { name: "bitrate".into(), display_name: "Bitrate (kbps)".into(), type_: PropertyType::Int, default: PropertyValue::Int(128), min: Some(64.0), max: Some(320.0), ..Default::default() },
            PropertyDef { name: "sample_rate".into(), display_name: "Sample Rate".into(), type_: PropertyType::Enum, default: PropertyValue::Enum("48000".into()), enum_values: vec![("44100".into(), "44.1 kHz".into()), ("48000".into(), "48 kHz".into())], ..Default::default() },
        ]
    }

    fn get_parameter(&self, name: &str) -> Option<PropertyValue> {
        match name {
            "bitrate" => Some(PropertyValue::Int(self.bitrate as i64)),
            "sample_rate" => Some(PropertyValue::Int(self.sample_rate as i64)),
            _ => None,
        }
    }

    fn set_parameter(&mut self, name: &str, value: PropertyValue) -> Result<()> {
        match name {
            "bitrate" => { if let PropertyValue::Int(b) = value { self.bitrate = b as u32; } }
            "sample_rate" => { if let PropertyValue::Int(s) = value { self.sample_rate = s as u32; } }
            _ => {}
        }
        Ok(())
    }

    async fn initialize(&mut self, input: MediaInfo, _output: MediaInfo) -> Result<()> {
        if let MediaInfo::Audio(a) = input {
            let info = &a.audio;
            self.input_info = Some(info.clone());
            self.sample_rate = info.sample_rate;
            self.channels = info.channels() as u32;

            let sr_str = self.sample_rate.to_string();
            let ch_str = self.channels.to_string();
            let br_str = format!("{}k", self.bitrate);

            let mut cmd = FfmpegCommand::new();
            cmd.args(["-f", "f32le"]);
            cmd.args(["-ar", &sr_str]);
            cmd.args(["-ac", &ch_str]);
            cmd.input("-");
            cmd.args(["-c:a", "aac"]);
            cmd.args(["-b:a", &br_str]);
            cmd.args(["-f", "adts"]);
            cmd.output("-");

            let mut child = cmd.spawn()?;
            self.stdin = child.take_stdin();
            self.child = Some(child);
            self.initialized = true;

            println!("[FFmpeg AAC] Initialized: {}Hz, {}ch, {}kbps",
                self.sample_rate, self.channels, self.bitrate);
        }
        Ok(())
    }

    async fn encode(&mut self, input: MediaData) -> Result<Option<EncodedPacket>> {
        if !self.initialized { return Ok(None); }
        if let MediaData::Audio(frame) = input {
            if let Some(ref mut stdin) = self.stdin {
                stdin.write_all(&frame.data)?;
                stdin.flush()?;
            }
            if let Some(ref mut child) = self.child {
                if let Ok(mut iter) = child.iter() {
                    for event in iter.by_ref() {
                        if let FfmpegEvent::OutputChunk(data) = event {
                            let packet = EncodedPacket {
                                data, pts: self.frame_count, dts: self.frame_count,
                                duration: 0, keyframe: true, track: TrackId(0),
                            };
                            self.frame_count += frame.frames as i64;
                            return Ok(Some(packet));
                        }
                    }
                }
            }
            self.frame_count += frame.frames as i64;
            Ok(None)
        } else { Ok(None) }
    }

    async fn flush(&mut self) -> Result<Vec<EncodedPacket>> {
        let mut packets = Vec::new();
        self.stdin = None;
        if let Some(mut child) = self.child.take() {
            if let Ok(mut iter) = child.iter() {
                for event in iter.by_ref() {
                    if let FfmpegEvent::OutputChunk(data) = event {
                        packets.push(EncodedPacket {
                            data, pts: self.frame_count, dts: self.frame_count,
                            duration: 0, keyframe: true, track: TrackId(0),
                        });
                    }
                }
            }
        }
        self.initialized = false;
        Ok(packets)
    }

    fn as_any(&self) -> &dyn Any { self }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

pub fn create_ffmpeg_aac_encoder() -> Box<dyn Encoder> {
    Box::new(FfmpegAacEncoder::new())
}
