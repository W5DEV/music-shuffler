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

struct MusicShuffler {
    music_directory: Option<PathBuf>,
    playlist: Vec<(PathBuf, SongMetadata)>,
    current_song_index: usize,
    music_files: Vec<PathBuf>,
    audio_player: Option<AudioPlayer>,
}

impl Default for MusicShuffler {
    fn default() -> Self {
        Self {
            music_directory: None,
            playlist: Vec::new(),
            current_song_index: 0,
            music_files: Vec::new(),
            audio_player: AudioPlayer::new().ok(),
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
                            return;
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

impl eframe::App for MusicShuffler {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
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
                    if ui.button("Generate Playlist").clicked() {
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
                        
                        // Load metadata lazily in a separate thread to avoid blocking UI
                        self.playlist.clear();
                        self.current_song_index = 0;
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
                        
                        // Load metadata with caching
                        println!("Loading metadata for {} tracks...", files.len());
                        let mut cache = load_file_cache().unwrap_or_else(|| FileCache {
                            directory: self.music_directory.as_ref().unwrap().clone(),
                            last_scan: SystemTime::now(),
                            files: self.music_files.clone(),
                            metadata_cache: HashMap::new(),
                        });
                        
                        let mut cache_updated = false;
                        for (i, path) in files.iter().enumerate() {
                            let mut metadata_loaded = false;
                            
                            // Try to load from cache first
                            if let Some(cached) = cache.metadata_cache.get(path) {
                                if let Some((file_size, modified_time)) = get_file_info(path) {
                                    if cached.file_size == file_size && cached.modified_time == modified_time {
                                        // Cache hit - use cached metadata
                                        self.playlist[i] = (path.clone(), cached.metadata.clone());
                                        metadata_loaded = true;
                                    }
                                }
                            }
                            
                            // If not in cache or file changed, load fresh
                            if !metadata_loaded {
                                if let Ok(mut metadata) = SongMetadata::from_path(path) {
                                    metadata.duration = extract_duration_symphonia(path);
                                    self.playlist[i] = (path.clone(), metadata.clone());
                                    
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
                                println!("Loaded metadata for {}/{} tracks", i + 1, files.len());
                            }
                        }
                        
                        // Save updated cache
                        if cache_updated {
                            save_file_cache(&cache);
                        }
                        
                        println!("Metadata loading complete!");
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
                            .show(ui, |ui| {
                                let indices_to_remove = Vec::new();
                                for (i, track) in self.playlist.iter().enumerate() {
                                let is_current = self.current_song_index == i;
                                let row_height = 24.0;
                                let padding_h = 4.0;
                                let (rect, _response) = ui.allocate_exact_size(egui::vec2(400.0, row_height), egui::Sense::hover());
                                if is_current {
                                    let painter = ui.painter();
                                    painter.rect_filled(rect, 4.0, egui::Color32::from_rgb(70, 120, 100));
                                }
                                ui.allocate_new_ui(egui::UiBuilder::new().max_rect(rect), |ui| {
                                    ui.horizontal(|ui| {
                                        ui.add_space(padding_h);
                                        let max_title_chars = 36;
                                        let title = &track.1.title;
                                        let display_title = if title.chars().count() > max_title_chars {
                                            let mut s = title.chars().take(max_title_chars - 1).collect::<String>();
                                            s.push('…');
                                            s
                                        } else {
                                            title.clone()
                                        };
                                        let rich_text = if is_current {
                                            egui::RichText::new(display_title).size(14.0).color(egui::Color32::WHITE)
                                        } else {
                                            egui::RichText::new(display_title).size(14.0)
                                        };
                                        let label_response = ui.label(rich_text);
                                        if label_response.clicked() {
                                            self.current_song_index = i;
                                            if let Err(e) = self.audio_player.as_mut().unwrap().play(&track.0) {
                                                eprintln!("Error playing track '{}': {}", track.1.title, e);
                                                eprintln!("This file may be corrupted. Try re-encoding or replacing it.");
                                            }
                                        }
                                        if label_response.hovered() {
                                            ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
                                            label_response.on_hover_ui(|ui| { ui.label(title); });
                                        }
                                        ui.add_space(padding_h);
                                    });
                                });
                                ui.add_space(4.0); // 4px gap below each item
                            }
                            for &index in indices_to_remove.iter().rev() {
                                self.playlist.remove(index);
                                if self.current_song_index >= self.playlist.len() {
                                    self.current_song_index = self.playlist.len().saturating_sub(1);
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
                        if let Some(art_data) = &metadata.album_art {
                            if let Ok(img) = image::load_from_memory(art_data) {
                                let size = [200, 200];
                                let img = img.resize_exact(size[0], size[1], image::imageops::FilterType::Lanczos3);
                                let img_buffer = img.to_rgb8();
                                let color_image = egui::ColorImage::from_rgb([
                                    size[0] as usize,
                                    size[1] as usize
                                ], &img_buffer);
                                let texture = ui.ctx().load_texture(
                                    "album_art",
                                    color_image,
                                    egui::TextureOptions::default(),
                                );
                                ui.add(egui::Image::new((texture.id(), egui::vec2(size[0] as f32, size[1] as f32))));
                            }
                        }
                        ui.label(format!("{}", metadata.title));
                        ui.label(format!("{}", metadata.artist));
                        ui.label(format!("{}", metadata.album));
                        // Progress bar and time
                        let (progress, duration_secs) = if let Some(player) = &self.audio_player {
                            let player_duration = player.get_duration().map(|d| d.as_secs_f32());
                            
                            // Try to get duration from audio player first, then fallback to metadata
                            let duration_secs = player_duration
                                .filter(|&d| d > 0.0)
                                .or(metadata.duration)
                                .unwrap_or(0.0);
                            
                            // Use the appropriate progress calculation based on available duration
                            let progress = if duration_secs > 0.0 {
                                player.get_progress_with_duration(duration_secs).unwrap_or(0.0)
                            } else {
                                0.0
                            };
                            
                            (progress.clamp(0.0, 1.0), duration_secs)
                        } else {
                            (0.0, metadata.duration.unwrap_or(0.0))
                        };
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
                                if ui.add_sized([50.0, 50.0], egui::Button::new(egui::RichText::new("  ⏮  ").size(25.0).monospace().strong()).frame(true).min_size(egui::vec2(50.0, 50.0)).corner_radius(25.0)).clicked() {
                                    if self.current_song_index > 0 {
                                        self.current_song_index -= 1;
                                        if let Some((path, metadata)) = self.playlist.get(self.current_song_index) {
                                            if let Err(e) = self.audio_player.as_mut().unwrap().play(path) {
                                                eprintln!("Error playing track '{}': {}", metadata.title, e);
                                                eprintln!("This file may be corrupted. Try re-encoding or replacing it.");
                                            }
                                        }
                                    }
                                }
                                let play_symbol = if self.audio_player.as_ref().unwrap().is_playing() { "  ⏸  " } else { "  ▶  " };
                                if ui.add_sized([75.0, 75.0], egui::Button::new(egui::RichText::new(play_symbol).size(37.0).monospace().strong()).frame(true).min_size(egui::vec2(75.0, 75.0)).corner_radius(37.5)).clicked() {
                                    if self.audio_player.as_ref().unwrap().is_playing() {
                                        self.audio_player.as_mut().unwrap().pause();
                                    } else {
                                        if let Some((path, metadata)) = self.playlist.get(self.current_song_index) {
                                            if let Err(e) = self.audio_player.as_mut().unwrap().play(path) {
                                                eprintln!("Error playing track '{}': {}", metadata.title, e);
                                                eprintln!("This file may be corrupted. Try re-encoding or replacing it.");
                                            }
                                        } else if !self.playlist.is_empty() {
                                            self.current_song_index = 0;
                                            if let Some((path, metadata)) = self.playlist.get(0) {
                                                if let Err(e) = self.audio_player.as_mut().unwrap().play(path) {
                                                    eprintln!("Error playing track '{}': {}", metadata.title, e);
                                                    eprintln!("This file may be corrupted. Try re-encoding or replacing it.");
                                                }
                                            }
                                        }
                                    }
                                }
                                if ui.add_sized([50.0, 50.0], egui::Button::new(egui::RichText::new("  ⏭  ").size(25.0).monospace().strong()).frame(true).min_size(egui::vec2(50.0, 50.0)).corner_radius(25.0)).clicked() {
                                    if self.current_song_index < self.playlist.len() - 1 {
                                        self.current_song_index += 1;
                                        if let Some((path, metadata)) = self.playlist.get(self.current_song_index) {
                                            if let Err(e) = self.audio_player.as_mut().unwrap().play(path) {
                                                eprintln!("Error playing track '{}': {}", metadata.title, e);
                                                eprintln!("This file may be corrupted. Try re-encoding or replacing it.");
                                            }
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