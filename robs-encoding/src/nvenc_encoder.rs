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

pub struct NvencH264Encoder {
    id: EncoderId,
    name: String,
    bitrate: u32,
    keyint: u32,
    rate_control: RateControlMode,
    preset: String,
    profile: String,
    gpu_index: u32,
    caps: EncoderCapabilities,
    stdin: Option<ChildStdin>,
    child: Option<FfmpegChild>,
    frame_count: i64,
    initialized: bool,
    input_info: Option<VideoInfo>,
    width: u32,
    height: u32,
    fps_num: u32,
    fps_den: u32,
    pixel_format: &'static str,
}

impl NvencH264Encoder {
    pub fn new() -> Self {
        Self {
            id: EncoderId(ObjectId::new()),
            name: "NVIDIA NVENC H.264".to_string(),
            bitrate: 6000,
            keyint: 2,
            rate_control: RateControlMode::CBR,
            preset: "p4".to_string(),
            profile: "high".to_string(),
            gpu_index: 0,
            caps: EncoderCapabilities {
                max_width: 4096, max_height: 2160, max_fps: 120,
                supports_hardware: true,
                supported_pixel_formats: vec![PixelFormat::NV12, PixelFormat::I420],
            },
            stdin: None, child: None, frame_count: 0, initialized: false,
            input_info: None, width: 1920, height: 1080,
            fps_num: 30, fps_den: 1, pixel_format: "nv12",
        }
    }

    pub fn is_available() -> bool {
        std::process::Command::new("ffmpeg")
            .args(["-codecs"])
            .output()
            .map(|o| String::from_utf8_lossy(&o.stdout).contains("h264_nvenc"))
            .unwrap_or(false)
    }

    pub fn get_gpu_count() -> u32 { if !Self::is_available() { 0 } else { 1 } }
}

impl Default for NvencH264Encoder { fn default() -> Self { Self::new() } }

#[async_trait]
impl Encoder for NvencH264Encoder {
    fn id(&self) -> EncoderId { self.id }
    fn name(&self) -> &str { &self.name }
    fn codec_name(&self) -> &str { "h264" }
    fn media_type(&self) -> MediaType { MediaType::Video }
    fn get_input_info(&self) -> Option<MediaInfo> {
        self.input_info.as_ref().map(|v| MediaInfo::Video(VideoEncodeInfo {
            video: v.clone(), encoder_name: "h264_nvenc".to_string(),
            codec_params: serde_json::json!({}),
        }))
    }
    fn get_output_info(&self) -> Option<MediaInfo> { None }
    fn get_caps(&self) -> EncoderCapabilities { self.caps.clone() }

    fn get_presets(&self) -> Vec<EncoderPreset> {
        vec![
            EncoderPreset { id: 0, name: "p1".into(), description: "Fastest".into(), quality_level: 1, speed_level: 7 },
            EncoderPreset { id: 1, name: "p2".into(), description: "Faster".into(), quality_level: 2, speed_level: 6 },
            EncoderPreset { id: 2, name: "p3".into(), description: "Fast".into(), quality_level: 3, speed_level: 5 },
            EncoderPreset { id: 3, name: "p4".into(), description: "Medium".into(), quality_level: 4, speed_level: 4 },
            EncoderPreset { id: 4, name: "p5".into(), description: "Slow".into(), quality_level: 5, speed_level: 3 },
            EncoderPreset { id: 5, name: "p6".into(), description: "Slower".into(), quality_level: 6, speed_level: 2 },
            EncoderPreset { id: 6, name: "p7".into(), description: "Slowest".into(), quality_level: 7, speed_level: 1 },
        ]
    }
    fn get_current_preset(&self) -> EncoderPreset {
        self.get_presets().into_iter().find(|p| p.name == self.preset)
            .unwrap_or_else(|| EncoderPreset { id: 3, name: "p4".into(), description: "Medium".into(), quality_level: 4, speed_level: 4 })
    }
    fn set_preset(&mut self, preset: EncoderPreset) -> Result<()> { self.preset = preset.name; Ok(()) }

    fn parameters_definition(&self) -> Vec<PropertyDef> {
        vec![
            PropertyDef { name: "bitrate".into(), display_name: "Bitrate (kbps)".into(), type_: PropertyType::Int, default: PropertyValue::Int(6000), min: Some(100.0), max: Some(50000.0), ..Default::default() },
            PropertyDef { name: "keyint".into(), display_name: "Keyframe Interval (seconds)".into(), type_: PropertyType::Int, default: PropertyValue::Int(2), min: Some(1.0), max: Some(20.0), ..Default::default() },
            PropertyDef { name: "rate_control".into(), display_name: "Rate Control".into(), type_: PropertyType::Enum, default: PropertyValue::Enum("CBR".into()), enum_values: vec![("CBR".into(), "CBR".into()), ("VBR".into(), "VBR".into()), ("CQP".into(), "CQP".into())], ..Default::default() },
            PropertyDef { name: "profile".into(), display_name: "H.264 Profile".into(), type_: PropertyType::Enum, default: PropertyValue::Enum("high".into()), enum_values: vec![("baseline".into(), "Baseline".into()), ("main".into(), "Main".into()), ("high".into(), "High".into())], ..Default::default() },
            PropertyDef { name: "gpu".into(), display_name: "GPU Index".into(), type_: PropertyType::Int, default: PropertyValue::Int(0), min: Some(0.0), max: Some(7.0), ..Default::default() },
        ]
    }

    fn get_parameter(&self, name: &str) -> Option<PropertyValue> {
        match name {
            "bitrate" => Some(PropertyValue::Int(self.bitrate as i64)),
            "keyint" => Some(PropertyValue::Int(self.keyint as i64)),
            "rate_control" => Some(PropertyValue::Enum(format!("{:?}", self.rate_control))),
            "profile" => Some(PropertyValue::Enum(self.profile.clone())),
            "gpu" => Some(PropertyValue::Int(self.gpu_index as i64)),
            _ => None,
        }
    }

    fn set_parameter(&mut self, name: &str, value: PropertyValue) -> Result<()> {
        match name {
            "bitrate" => { if let PropertyValue::Int(b) = value { self.bitrate = b as u32; } }
            "keyint" => { if let PropertyValue::Int(k) = value { self.keyint = k as u32; } }
            "rate_control" => { if let PropertyValue::Enum(rc) = value { self.rate_control = match rc.as_str() { "CBR" => RateControlMode::CBR, "VBR" => RateControlMode::VBR, "CQP" => RateControlMode::CQP, _ => RateControlMode::CBR }; } }
            "profile" => { if let PropertyValue::Enum(p) = value { self.profile = p; } }
            "gpu" => { if let PropertyValue::Int(g) = value { self.gpu_index = g as u32; } }
            _ => {}
        }
        Ok(())
    }

    async fn initialize(&mut self, input: MediaInfo, _output: MediaInfo) -> Result<()> {
        if let MediaInfo::Video(v) = input {
            let info = &v.video;
            self.input_info = Some(info.clone());
            self.width = info.width;
            self.height = info.height;
            self.fps_num = info.fps_num;
            self.fps_den = info.fps_den;
            self.pixel_format = match info.format {
                PixelFormat::NV12 => "nv12",
                PixelFormat::I420 => "yuv420p",
                _ => "nv12",
            };

            let size_str = format!("{}x{}", self.width, self.height);
            let fps_str = format!("{}/{}", self.fps_num, self.fps_den);
            let br_str = format!("{}k", self.bitrate);
            let br2_str = format!("{}k", self.bitrate * 2);
            let keyint = self.keyint * self.fps_num / self.fps_den;
            let keyint_str = keyint.to_string();
            let keyint_min_str = self.keyint.to_string();
            let gpu_str = self.gpu_index.to_string();

            let mut cmd = FfmpegCommand::new();
            cmd.args(["-f", "rawvideo"]);
            cmd.args(["-pix_fmt", self.pixel_format]);
            cmd.args(["-s", &size_str]);
            cmd.args(["-r", &fps_str]);
            cmd.input("-");
            cmd.args(["-c:v", "h264_nvenc"]);
            cmd.args(["-preset", &self.preset]);
            cmd.args(["-profile:v", &self.profile]);
            cmd.args(["-tune", "ll"]);
            cmd.args(["-gpu", &gpu_str]);

            match self.rate_control {
                RateControlMode::CBR => {
                    cmd.args(["-b:v", &br_str]);
                    cmd.args(["-maxrate", &br_str]);
                    cmd.args(["-bufsize", &br2_str]);
                    cmd.args(["-rc", "cbr"]);
                }
                RateControlMode::VBR => {
                    cmd.args(["-b:v", &br_str]);
                    cmd.args(["-rc", "vbr"]);
                }
                RateControlMode::CQP => {
                    cmd.args(["-rc", "constqp"]);
                    cmd.args(["-cq", "23"]);
                }
                _ => {}
            }

            cmd.args(["-g", &keyint_str]);
            cmd.args(["-keyint_min", &keyint_min_str]);
            cmd.args(["-f", "h264"]);
            cmd.output("-");

            let mut child = cmd.spawn()?;
            self.stdin = child.take_stdin();
            self.child = Some(child);
            self.initialized = true;

            println!("[NVENC H.264] Initialized: {}x{} @ {}fps, {}kbps, preset={}",
                self.width, self.height, self.fps_num / self.fps_den, self.bitrate, self.preset);
        }
        Ok(())
    }

    async fn encode(&mut self, input: MediaData) -> Result<Option<EncodedPacket>> {
        if !self.initialized { return Ok(None); }
        if let MediaData::Video(frame) = input {
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
                                duration: 0, keyframe: self.frame_count == 0, track: TrackId(0),
                            };
                            self.frame_count += 1;
                            return Ok(Some(packet));
                        }
                    }
                }
            }
            self.frame_count += 1;
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
                            duration: 0, keyframe: false, track: TrackId(0),
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

pub fn create_nvenc_h264_encoder() -> Box<dyn Encoder> {
    Box::new(NvencH264Encoder::new())
}
