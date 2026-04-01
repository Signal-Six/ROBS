use robs_core::*;

pub struct AudioMixer {
    sample_rate: u32,
    channels: usize,
    mixer_buffer: Vec<f32>,
}

impl AudioMixer {
    pub fn new(sample_rate: u32, channels: usize) -> Self {
        Self {
            sample_rate,
            channels,
            mixer_buffer: Vec::new(),
        }
    }
    
    pub fn mix(&mut self, inputs: &[&AudioFrame], output_frames: u32) -> AudioFrame {
        let output_channels = self.channels;
        let sample_count = (output_frames as usize) * output_channels;
        
        if self.mixer_buffer.len() < sample_count {
            self.mixer_buffer.resize(sample_count, 0.0);
        }
        
        for sample in self.mixer_buffer.iter_mut() {
            *sample = 0.0;
        }
        
        for input in inputs {
            let input_samples = unsafe {
                std::slice::from_raw_parts(
                    input.data.as_ptr() as *const f32,
                    (input.frames as usize) * input.speakers.len()
                )
            };
            
            for (i, &sample) in input_samples.iter().enumerate() {
                if i < self.mixer_buffer.len() {
                    self.mixer_buffer[i] += sample;
                }
            }
        }
        
        let max_sample = self.mixer_buffer.iter()
            .map(|&s| s.abs())
            .fold(0.0f32, |a, b| a.max(b));
        
        if max_sample > 1.0 {
            let scale = 1.0 / max_sample;
            for sample in self.mixer_buffer.iter_mut() {
                *sample *= scale;
            }
        }
        
        let mut output = AudioFrame::new(output_frames, &AudioInfo {
            sample_rate: self.sample_rate,
            format: AudioFormat::F32,
            speakers: vec![AudioSpeaker::FL, AudioSpeaker::FR],
        });
        
        let output_samples = unsafe {
            std::slice::from_raw_parts_mut(
                output.data.as_mut_ptr() as *mut f32,
                sample_count
            )
        };
        output_samples.copy_from_slice(&self.mixer_buffer[..sample_count]);
        
        output
    }
}