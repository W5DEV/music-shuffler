#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod music;
mod audio;
mod metadata;

use eframe::egui;
use std::path::PathBuf;
use rfd::FileDialog;
use audio::AudioPlayer;
use metadata::SongMetadata;
use directories::ProjectDirs;
use symphonia::core::probe::Hint;
use symphonia::default::get_probe;
use std::fs::File;
use symphonia::core::io::MediaSourceStream;
use rodio::{Decoder, Source};
use std::collections::HashMap;
use std::time::SystemTime;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};
use std::thread;

struct MusicShuffler {
    music_directory: Option<PathBuf>,
    playlist: Vec<(PathBuf, SongMetadata)>,
    current_song_index: usize,
    music_files: Vec<PathBuf>,
    audio_player: Option<AudioPlayer>,
    metadata_loading: bool,
    pending_metadata: Arc<Mutex<Vec<(usize, PathBuf, SongMetadata)>>>,
    last_metadata_check: SystemTime,
    cached_progress: f32,
    cached_duration: f32,
    last_progress_update: SystemTime,
}

impl Default for MusicShuffler {
    fn default() -> Self {
        Self {
            music_directory: None,
            playlist: Vec::new(),
            current_song_index: 0,
            music_files: Vec::new(),
            audio_player: AudioPlayer::new().ok(),
            metadata_loading: false,
            pending_metadata: Arc::new(Mutex::new(Vec::new())),
            last_metadata_check: SystemTime::now(),
            cached_progress: 0.0,
            cached_duration: 0.0,
            last_progress_update: SystemTime::now(),
        }
    }
}

fn get_config_path() -> Option<std::path::PathBuf> {
    ProjectDirs::from("com", "yourorg", "music-shuffler")
        .map(|proj_dirs| proj_dirs.config_dir().join("config.txt"))
}

// Cache entry for file metadata
#[derive(Serialize, Deserialize, Clone)]
struct CachedMetadata {
    metadata: metadata::SongMetadata,
    file_size: u64,
    modified_time: SystemTime,
}

// Cache for scanned file list
#[derive(Serialize, Deserialize)]
struct FileCache {
    directory: std::path::PathBuf,
    last_scan: SystemTime,
    files: Vec<std::path::PathBuf>,
    metadata_cache: HashMap<std::path::PathBuf, CachedMetadata>,
}

// Simple file-based cache for metadata
fn get_cache_file_path() -> Option<std::path::PathBuf> {
    get_config_path().map(|p| p.parent().unwrap().join("file_cache.json"))
}

fn load_file_cache() -> Option<FileCache> {
    if let Some(cache_path) = get_cache_file_path() {
        if let Ok(contents) = std::fs::read_to_string(cache_path) {
            if let Ok(cache) = serde_json::from_str(&contents) {
                return Some(cache);
            }
        }
    }
    None
}

fn save_file_cache(cache: &FileCache) {
    if let Some(cache_path) = get_cache_file_path() {
        if let Some(parent) = cache_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Ok(contents) = serde_json::to_string(cache) {
            let _ = std::fs::write(cache_path, contents);
            println!("Cache saved with {} files and {} metadata entries", 
                     cache.files.len(), cache.metadata_cache.len());
        }
    }
}

fn is_cache_valid(cache: &FileCache, current_dir: &std::path::Path) -> bool {
    // Check if directory matches
    if cache.directory != current_dir {
        return false;
    }
    
    // Check if cache is not too old (optional - could scan daily)
    // For now, trust the cache until the directory changes
    
    // Could add more sophisticated validation here
    true
}

fn get_file_info(path: &std::path::Path) -> Option<(u64, SystemTime)> {
    if let Ok(metadata) = std::fs::metadata(path) {
        if let Ok(modified) = metadata.modified() {
            return Some((metadata.len(), modified));
        }
    }
    None
}

impl MusicShuffler {
    fn save_directory(&self) {
        if let Some(dir) = &self.music_directory {
            if let Some(config_path) = get_config_path() {
                let _ = std::fs::create_dir_all(config_path.parent().unwrap());
                let _ = std::fs::write(config_path, dir.to_string_lossy().to_string());
            }
        }
    }
    fn load_directory(&mut self) {
        if let Some(config_path) = get_config_path() {
            if let Ok(contents) = std::fs::read_to_string(config_path) {
                let path = std::path::PathBuf::from(contents.trim());
                if path.exists() && path.is_dir() {
                    self.music_directory = Some(path.clone());
                    
                    // Try to load from cache first
                    if let Some(cache) = load_file_cache() {
                        if is_cache_valid(&cache, &path) {
                            println!("Loading {} files from cache...", cache.files.len());
                            self.music_files = cache.files;
                            println!("Cache loaded successfully!");
                        } else {
                            println!("Cache invalid - directory changed");
                        }
                    } else {
                        println!("No cache found - will scan on first playlist generation");
                    }
                }
            }
        }
    }
}

fn format_time(secs: f32) -> String {
    let total_seconds = secs as u64;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    if hours > 0 {
        format!("{}:{:02}:{:02}", hours, minutes, seconds)
    } else {
        format!("{}:{:02}", minutes, seconds)
    }
}

fn extract_duration_symphonia(path: &std::path::Path) -> Option<f32> {
    // Try to extract duration using Symphonia
    if let Ok(file) = File::open(path) {
        let mss = MediaSourceStream::new(Box::new(file), Default::default());
        let mut hint = Hint::new();
        if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
            hint.with_extension(ext);
        }
        
        if let Ok(probe) = get_probe().format(
            &hint,
            mss,
            &Default::default(),
            &Default::default(),
        ) {
            let mut format = probe.format;
            
            // Try to get duration from metadata first
            if let Some(metadata_rev) = format.metadata().current() {
                for tag in metadata_rev.tags() {
                    if tag.key.as_str().to_lowercase() == "duration" {
                        if let Ok(duration_str) = tag.value.to_string().parse::<f32>() {
                            return Some(duration_str);
                        }
                    }
                }
            }
            
            // Try to calculate duration from track parameters
            if let Some(track) = format.default_track() {
                let params = &track.codec_params;
                
                // Method 1: Use n_frames and sample_rate
                if let (Some(n_frames), Some(sample_rate)) = (params.n_frames, params.sample_rate) {
                    return Some(n_frames as f32 / sample_rate as f32);
                }
                
                // Method 2: Use time_base and n_frames
                if let (Some(n_frames), Some(time_base)) = (params.n_frames, params.time_base) {
                    let duration_secs = n_frames as f64 * time_base.numer as f64 / time_base.denom as f64;
                    return Some(duration_secs as f32);
                }
            }
        }
    }
    
    // If Symphonia fails, try using rodio as a fallback
    if let Ok(file) = File::open(path) {
        let reader = std::io::BufReader::new(file);
        if let Ok(decoder) = Decoder::new(reader) {
            if let Some(duration) = decoder.total_duration() {
                return Some(duration.as_secs_f32());
            }
        }
    }
    
    None
}

impl MusicShuffler {
    fn check_pending_metadata(&mut self) {
        let updates = if let Ok(mut pending) = self.pending_metadata.try_lock() {
            let updates: Vec<_> = pending.drain(..).collect();
            updates
        } else {
            return;
        };

        if !updates.is_empty() {
            for (index, path, metadata) in updates {
                if index < self.playlist.len() {
                    self.playlist[index] = (path, metadata);
                }
            }
            // Check if all metadata is loaded
            let all_loaded = self.playlist.iter().all(|(_, meta)| meta.artist != "Loading...");
            if all_loaded {
                self.metadata_loading = false;
            }
        }
    }
}

impl eframe::App for MusicShuffler {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Only check metadata every 200ms to avoid constant updates
        if self.last_metadata_check.elapsed().unwrap_or_default().as_millis() > 200 {
            self.check_pending_metadata();
            self.last_metadata_check = SystemTime::now();
        }
        
        // Update progress cache only occasionally
        if self.last_progress_update.elapsed().unwrap_or_default().as_millis() > 100 {
            if let Some(player) = &self.audio_player {
                if let Some((_, metadata)) = self.playlist.get(self.current_song_index) {
                    let duration_secs = metadata.duration.unwrap_or(0.0);
                    if duration_secs > 0.0 {
                        self.cached_progress = player.get_progress_with_duration(duration_secs).unwrap_or(0.0).clamp(0.0, 1.0);
                        self.cached_duration = duration_secs;
                    }
                }
            }
            self.last_progress_update = SystemTime::now();
        }
        
        // Auto-advance to next song when current song finishes
        if let Some(player) = &self.audio_player {
            if player.has_finished() && !self.playlist.is_empty() {
                // Move to next song
                if self.current_song_index < self.playlist.len() - 1 {
                    self.current_song_index += 1;
                    if let Some((path, metadata)) = self.playlist.get(self.current_song_index) {
                        if let Err(e) = self.audio_player.as_mut().unwrap().play(path) {
                            eprintln!("Error playing next track '{}': {}", metadata.title, e);
                            eprintln!("This file may be corrupted. Try re-encoding or replacing it.");
                        }
                    }
                } else {
                    // Reached end of playlist - optionally loop back to beginning
                    self.current_song_index = 0;
                    if let Some((path, metadata)) = self.playlist.first() {
                        if let Err(e) = self.audio_player.as_mut().unwrap().play(path) {
                            eprintln!("Error playing first track '{}': {}", metadata.title, e);
                            eprintln!("This file may be corrupted. Try re-encoding or replacing it.");
                        }
                    }
                }
            }
        }
        
        // Update every 1 second, plus immediately on mouse input when paused
        ctx.request_repaint_after(std::time::Duration::from_secs(1));
        
        // Also respond to mouse when paused for good UX
        if let Some(player) = &self.audio_player {
            if !player.is_playing() {
                ctx.request_repaint_after(std::time::Duration::from_millis(16)); // ~60fps for responsiveness
            }
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            // Header
            ui.vertical_centered(|ui| {
                ui.heading("Music Shuffler");
                let dir_label = if let Some(dir) = &self.music_directory {
                    let dir_str = dir.display().to_string();
                    let max_chars = 60;
                    if dir_str.len() > max_chars {
                        format!("Current Directory: ...{}", &dir_str[dir_str.len()-max_chars..])
                    } else {
                        format!("Current Directory: {}", dir_str)
                    }
                } else {
                    "Select a music directory to get started".to_string()
                };
                ui.label(dir_label);
                ui.add_space(4.0);
                ui.horizontal(|ui| {
                    if ui.button("Select Directory").clicked() {
                        if let Some(path) = FileDialog::new().pick_folder() {
                            self.music_directory = Some(path.clone());
                            self.save_directory();
                            if let Ok(files) = music::scan_music_directory(&path) {
                                self.music_files = files;
                            }
                        }
                    }
                    if ui.button("Generate Playlist").clicked() && !self.metadata_loading {
                        // First scan directory if not already done
                        if self.music_files.is_empty() {
                            if let Some(dir) = &self.music_directory {
                                println!("Scanning directory for the first time...");
                                if let Ok(files) = music::scan_music_directory(dir) {
                                    self.music_files = files.clone();
                                    println!("Scan complete! Found {} music files", self.music_files.len());
                                    
                                    // Save to cache for next time
                                    let cache = FileCache {
                                        directory: dir.clone(),
                                        last_scan: SystemTime::now(),
                                        files,
                                        metadata_cache: HashMap::new(),
                                    };
                                    save_file_cache(&cache);
                                } else {
                                    println!("Failed to scan directory");
                                    return;
                                }
                            } else {
                                println!("No directory selected");
                                return;
                            }
                        }
                        
                        println!("Generating playlist...");
                        let files = music::generate_playlist(&self.music_files, 50);
                        
                        // Clear previous playlist and reset state
                        self.playlist.clear();
                        self.current_song_index = 0;
                        self.metadata_loading = true;
                        if let Some(player) = &mut self.audio_player {
                            player.stop();
                        }
                        
                        // Add files with placeholder metadata first for immediate display
                        for file in &files {
                            let placeholder_metadata = SongMetadata {
                                title: file.file_stem()
                                    .map(|s| s.to_string_lossy().to_string())
                                    .unwrap_or_else(|| "Unknown".to_string()),
                                artist: "Loading...".to_string(),
                                album: "Loading...".to_string(),
                                duration: None,
                                album_art: None,
                            };
                            self.playlist.push((file.clone(), placeholder_metadata));
                        }
                        
                        // Set loading flag
                        self.metadata_loading = true;
                        
                        // Load metadata in background thread
                        let files_for_bg = files.clone();
                        let pending_metadata = Arc::clone(&self.pending_metadata);
                        let music_dir = self.music_directory.clone().unwrap();
                        
                        thread::spawn(move || {
                            println!("Loading metadata for {} tracks in background...", files_for_bg.len());
                            let mut cache = load_file_cache().unwrap_or_else(|| FileCache {
                                directory: music_dir,
                                last_scan: SystemTime::now(),
                                files: files_for_bg.clone(),
                                metadata_cache: HashMap::new(),
                            });
                            
                            let mut cache_updated = false;
                            for (i, path) in files_for_bg.iter().enumerate() {
                                let mut metadata_loaded = false;
                                
                                // Try to load from cache first
                                if let Some(cached) = cache.metadata_cache.get(path) {
                                    if let Some((file_size, modified_time)) = get_file_info(path) {
                                        if cached.file_size == file_size && cached.modified_time == modified_time {
                                            // Cache hit - use cached metadata
                                            if let Ok(mut pending) = pending_metadata.lock() {
                                                pending.push((i, path.clone(), cached.metadata.clone()));
                                            }
                                            metadata_loaded = true;
                                        }
                                    }
                                }
                                
                                // If not in cache or file changed, load fresh
                                if !metadata_loaded {
                                    if let Ok(mut metadata) = SongMetadata::from_path(path) {
                                        metadata.duration = extract_duration_symphonia(path);
                                        
                                        if let Ok(mut pending) = pending_metadata.lock() {
                                            pending.push((i, path.clone(), metadata.clone()));
                                        }
                                        
                                        // Update cache
                                        if let Some((file_size, modified_time)) = get_file_info(path) {
                                            cache.metadata_cache.insert(path.clone(), CachedMetadata {
                                                metadata,
                                                file_size,
                                                modified_time,
                                            });
                                            cache_updated = true;
                                        }
                                    }
                                }
                                
                                if i % 10 == 0 {
                                    println!("Loaded metadata for {}/{} tracks", i + 1, files_for_bg.len());
                                }
                            }
                            
                            // Save updated cache
                            if cache_updated {
                                save_file_cache(&cache);
                            }
                            
                            println!("Background metadata loading complete!");
                        });
                    }
                });
            });
            ui.separator();
            // Main content: two fixed-width panels (400px each)
            ui.horizontal_top(|ui| {
                // Playlist panel (fixed 400px)
                ui.vertical(|ui| {
                    ui.set_width(400.0);
                    ui.heading("Playlist");
                    
                    if self.playlist.is_empty() {
                        ui.vertical_centered(|ui| {
                            ui.add_space(50.0);
                            ui.label("No playlist loaded");
                            ui.add_space(10.0);
                            if self.music_directory.is_some() {
                                ui.label("Click 'Generate Playlist' to create one");
                            } else {
                                ui.label("Select a directory first");
                            }
                        });
                    } else {
                        let available_height = ui.available_height();
                                                egui::ScrollArea::vertical()
                            .max_height(available_height)
                            .show_rows(ui, 20.0, self.playlist.len(), |ui, row_range| {
                                for i in row_range {
                                    if let Some((_, metadata)) = self.playlist.get(i) {
                                        let is_current = self.current_song_index == i;
                                        
                                        let response = ui.selectable_label(is_current, &metadata.title);
                                        
                                        if response.clicked() {
                                            self.current_song_index = i;
                                            if let Some(ref mut player) = self.audio_player {
                                                                                             if let Err(_e) = player.play(&self.playlist[i].0) {
                                                 eprintln!("Error playing track");
                                                }
                                            }
                                        }
                                    }
                                }
                            });
                    }
                });
                // Now Playing panel (fixed 400px)
                ui.vertical_centered(|ui| {
                    ui.set_width(400.0);
                    ui.heading("Now Playing");
                    if let Some((_path, metadata)) = self.playlist.get(self.current_song_index) {
                        // Simple grey square placeholder for album art
                        let (rect, _) = ui.allocate_exact_size(egui::vec2(200.0, 200.0), egui::Sense::hover());
                        ui.painter().rect_filled(rect, 8.0, egui::Color32::from_gray(128));
                        ui.label(metadata.title.to_string());
                        ui.label(metadata.artist.to_string());
                        ui.label(metadata.album.to_string());
                        // Progress bar and time (use cached values)
                        let (progress, duration_secs) = (self.cached_progress, self.cached_duration);
                                                // Simple progress bar (read-only)
                        let progress_bar = egui::ProgressBar::new(progress);
                        ui.add_sized([375.0, 20.0], progress_bar);
                        let current_secs = progress * duration_secs;
                        ui.label(format!("{} / {}", format_time(current_secs), format_time(duration_secs)));
                    }
                    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
                        ui.add_space(16.0);
                        let button_row_width = 400.0;
                        ui.add_space(40.0); // left padding
                        ui.allocate_ui_with_layout(
                            egui::vec2(button_row_width - 80.0, 75.0),
                            egui::Layout::left_to_right(egui::Align::Center),
                            |ui| {
                                if ui.add_sized([50.0, 50.0], egui::Button::new(egui::RichText::new("  ⏮  ").size(25.0).monospace().strong()).frame(true).min_size(egui::vec2(50.0, 50.0)).corner_radius(25.0)).clicked() && self.current_song_index > 0 {
                                    self.current_song_index -= 1;
                                    if let Some((path, metadata)) = self.playlist.get(self.current_song_index) {
                                        if let Err(e) = self.audio_player.as_mut().unwrap().play(path) {
                                            eprintln!("Error playing track '{}': {}", metadata.title, e);
                                            eprintln!("This file may be corrupted. Try re-encoding or replacing it.");
                                        }
                                    }
                                }
                                let play_symbol = if self.audio_player.as_ref().unwrap().is_playing() { "  ⏸  " } else { "  ▶  " };
                                if ui.add_sized([75.0, 75.0], egui::Button::new(egui::RichText::new(play_symbol).size(37.0).monospace().strong()).frame(true).min_size(egui::vec2(75.0, 75.0)).corner_radius(37.5)).clicked() {
                                    if self.audio_player.as_ref().unwrap().is_playing() {
                                        self.audio_player.as_mut().unwrap().pause();
                                    } else if let Some((path, metadata)) = self.playlist.get(self.current_song_index) {
                                        if let Err(e) = self.audio_player.as_mut().unwrap().play(path) {
                                            eprintln!("Error playing track '{}': {}", metadata.title, e);
                                            eprintln!("This file may be corrupted. Try re-encoding or replacing it.");
                                        }
                                    } else if !self.playlist.is_empty() {
                                        self.current_song_index = 0;
                                        if let Some((path, metadata)) = self.playlist.first() {
                                            if let Err(e) = self.audio_player.as_mut().unwrap().play(path) {
                                                eprintln!("Error playing track '{}': {}", metadata.title, e);
                                                eprintln!("This file may be corrupted. Try re-encoding or replacing it.");
                                            }
                                        }
                                    }
                                }
                                if ui.add_sized([50.0, 50.0], egui::Button::new(egui::RichText::new("  ⏭  ").size(25.0).monospace().strong()).frame(true).min_size(egui::vec2(50.0, 50.0)).corner_radius(25.0)).clicked() && self.current_song_index < self.playlist.len() - 1 {
                                    self.current_song_index += 1;
                                    if let Some((path, metadata)) = self.playlist.get(self.current_song_index) {
                                        if let Err(e) = self.audio_player.as_mut().unwrap().play(path) {
                                            eprintln!("Error playing track '{}': {}", metadata.title, e);
                                            eprintln!("This file may be corrupted. Try re-encoding or replacing it.");
                                        }
                                    }
                                }
                            }
                        );
                        ui.add_space(40.0); // right padding
                    });
                });
            });
        });
    }
}

fn main() {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0]),
        ..Default::default()
    };
    
    eframe::run_native(
        "Music Shuffler",
        options,
        Box::new(|_cc| {
            let mut app = MusicShuffler::default();
            app.load_directory();
            Ok(Box::new(app))
        }),
    ).unwrap();
} 