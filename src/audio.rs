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
    }

    pub fn pause(&mut self) {
        if let Some(sink) = &self.sink {
            sink.pause();
        }
    }

    pub fn is_playing(&self) -> bool {
        if let Some(sink) = &self.sink {
            !sink.is_paused() && !sink.empty()
        } else {
            false
        }
    }



    pub fn get_progress_with_duration(&self, total_duration_secs: f32) -> Option<f32> {
        if let Some(start_time) = self.start_time {
            if total_duration_secs > 0.0 {
                let elapsed = start_time.elapsed();
                let progress = elapsed.as_secs_f32() / total_duration_secs;
                Some(progress.clamp(0.0, 1.0))
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_duration(&self) -> Option<Duration> {
        self.duration
    }
} 