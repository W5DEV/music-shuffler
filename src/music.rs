use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::Result;
use rand::seq::SliceRandom;
use std::sync::{Arc, Mutex};
use std::thread;
use std::sync::mpsc;

pub fn scan_music_directory(dir: &Path) -> Result<Vec<PathBuf>> {
    scan_music_directory_fast(dir)
}

// Scan with progress callback
pub fn scan_music_directory_with_progress<F>(dir: &Path, progress_callback: F) -> Result<Vec<PathBuf>>
where
    F: Fn(String) + Send + Sync + 'static,
{
    let progress_callback = Arc::new(progress_callback);
    
    progress_callback("Discovering files...".to_string());
    
    // Collect all entries first (this is usually fast)
    let entries: Vec<_> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();
    
    progress_callback(format!("Found {} total files, filtering for music files...", entries.len()));
    
    let mut music_files = Vec::new();
    for (i, entry) in entries.iter().enumerate() {
        let path = entry.path();
        if is_music_file(path) {
            music_files.push(path.to_path_buf());
        }
        
        // Update progress every 100 files or so
        if i % 100 == 0 || i == entries.len() - 1 {
            progress_callback(format!("Processed {}/{} files, found {} music files", i + 1, entries.len(), music_files.len()));
        }
    }
    
    music_files.sort(); // Sort for consistent ordering
    progress_callback(format!("Scan complete! Found {} music files", music_files.len()));
    
    Ok(music_files)
}

// Fast scanning - just finds music files without loading metadata
pub fn scan_music_directory_fast(dir: &Path) -> Result<Vec<PathBuf>> {
    let _music_files = Arc::new(Mutex::new(Vec::<PathBuf>::new()));
    let (tx, rx) = mpsc::channel();
    
    // Use multiple threads for file scanning
    let num_threads = std::thread::available_parallelism()
        .map(|p| p.get())
        .unwrap_or(4)
        .min(8); // Cap at 8 threads to avoid overwhelming I/O
    
    println!("Scanning music directory with {} threads...", num_threads);
    
    // Collect all entries first (this is usually fast)
    let entries: Vec<_> = WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .collect();
    
    println!("Found {} total files, filtering for music files...", entries.len());
    
    // Split entries among threads
    let chunk_size = entries.len().div_ceil(num_threads);
    let mut handles = Vec::new();
    
    for chunk in entries.chunks(chunk_size) {
        let chunk = chunk.to_vec();
        let tx = tx.clone();
        
        let handle = thread::spawn(move || {
            let mut local_music_files = Vec::new();
            
            for entry in chunk {
                let path = entry.path();
                if is_music_file(path) {
                    local_music_files.push(path.to_path_buf());
                }
            }
            
            let _ = tx.send(local_music_files);
        });
        
        handles.push(handle);
    }
    
    // Drop the original sender
    drop(tx);
    
    // Collect results from all threads
    let mut all_music_files = Vec::new();
    for received_files in rx {
        all_music_files.extend(received_files);
    }
    
    // Wait for all threads to complete
    for handle in handles {
        let _ = handle.join();
    }
    
    all_music_files.sort(); // Sort for consistent ordering
    println!("Found {} music files", all_music_files.len());
    
    Ok(all_music_files)
}

fn is_music_file(path: &Path) -> bool {
    if let Some(ext) = path.extension() {
        let ext = ext.to_string_lossy().to_lowercase();
        matches!(ext.as_str(), "mp3" | "wav" | "ogg" | "flac" | "m4a" | "aac" | "wma")
    } else {
        false
    }
}

pub fn generate_playlist(music_files: &[PathBuf], count: usize) -> Vec<PathBuf> {
    let mut rng = rand::rng();
    let mut files_vec = music_files.to_vec();
    files_vec.shuffle(&mut rng);
    files_vec.into_iter().take(count).collect()
}

 