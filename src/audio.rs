use std::path::Path;
use rodio::{Decoder, OutputStream, Sink, Source};
use std::fs::File;
use std::io::BufReader;
use anyhow::Result;
use std::time::Duration;

pub struct AudioPlayer {
    sink: Option<Sink>,
    _stream: Option<OutputStream>,
    _stream_handle: Option<rodio::OutputStreamHandle>,
    duration: Option<Duration>,
    start_time: Option<std::time::Instant>,
    paused_time: Option<std::time::Instant>,
    total_paused_duration: Duration,
}

impl AudioPlayer {
    pub fn new() -> Result<Self> {
        let (stream, stream_handle) = OutputStream::try_default()?;
        Ok(Self {
            sink: None,
            _stream: Some(stream),
            _stream_handle: Some(stream_handle),
            duration: None,
            start_time: None,
            paused_time: None,
            total_paused_duration: Duration::ZERO,
        })
    }

    pub fn play(&mut self, path: &Path) -> Result<()> {
        // Stop any currently playing audio
        self.stop();

        // Create a new sink
        if let Some(handle) = &self._stream_handle {
            let sink = Sink::try_new(handle)?;
            
            // Open the file
            let file = File::open(path)?;
            let reader = BufReader::new(file);
            
            // Decode the file
            let decoder = Decoder::new(reader)?;
            
            // Store the duration
            self.duration = decoder.total_duration();
            
            // Add the decoder to the sink
            sink.append(decoder);
            
            // Store the sink and start time
            self.sink = Some(sink);
            self.start_time = Some(std::time::Instant::now());
            self.paused_time = None;
            self.total_paused_duration = Duration::ZERO;
        }

        Ok(())
    }

    pub fn stop(&mut self) {
        if let Some(sink) = &self.sink {
            sink.stop();
        }
        self.sink = None;
        self.duration = None;
        self.start_time = None;
        self.paused_time = None;
        self.total_paused_duration = Duration::ZERO;
    }

    pub fn pause(&mut self) {
        if let Some(sink) = &self.sink {
            if !sink.is_paused() {
                sink.pause();
                self.paused_time = Some(std::time::Instant::now());
            }
        }
    }
    
    pub fn resume(&mut self) {
        if let Some(sink) = &self.sink {
            if sink.is_paused() {
                sink.play();
                if let Some(paused_time) = self.paused_time {
                    self.total_paused_duration += paused_time.elapsed();
                    self.paused_time = None;
                }
            }
        }
    }

    pub fn is_playing(&self) -> bool {
        if let Some(sink) = &self.sink {
            !sink.is_paused() && !sink.empty()
        } else {
            false
        }
    }

    pub fn has_finished(&self) -> bool {
        if let Some(sink) = &self.sink {
            // Song has finished if sink exists but is empty (all sources consumed)
            sink.empty()
        } else {
            false
        }
    }

    pub fn is_paused(&self) -> bool {
        if let Some(sink) = &self.sink {
            sink.is_paused()
        } else {
            false
        }
    }

    pub fn get_progress_with_duration(&self, total_duration_secs: f32) -> Option<f32> {
        if let Some(start_time) = self.start_time {
            if total_duration_secs > 0.0 {
                let mut elapsed = start_time.elapsed();
                
                // Subtract total paused time
                elapsed = elapsed.saturating_sub(self.total_paused_duration);
                
                // If currently paused, don't add the current pause time
                if let Some(paused_time) = self.paused_time {
                    // We're currently paused, so don't add time since pause started
                    elapsed = elapsed.saturating_sub(paused_time.elapsed());
                }
                
                let progress = elapsed.as_secs_f32() / total_duration_secs;
                Some(progress.clamp(0.0, 1.0))
            } else {
                None
            }
        } else {
            None
        }
    }
} 