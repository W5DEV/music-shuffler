use std::path::Path;
use anyhow::Result;
use id3::{Tag, TagLike};
use metaflac::Tag as FlacTag;

#[derive(Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SongMetadata {
    pub title: String,
    pub artist: String,
    pub album: String,
    pub duration: Option<f32>,
    pub album_art: Option<Vec<u8>>,
}

impl SongMetadata {
    pub fn from_path(path: &Path) -> Result<Self> {
        let mut metadata = SongMetadata::default();
        
        // Get file name as default title
        if let Some(file_name) = path.file_stem() {
            metadata.title = file_name.to_string_lossy().to_string();
        }

        // Try to get metadata based on file extension
        if let Some(ext) = path.extension() {
            match ext.to_string_lossy().to_lowercase().as_str() {
                "mp3" => {
                    if let Ok(tag) = Tag::read_from_path(path) {
                        metadata.title = tag.title().unwrap_or(&metadata.title).to_string();
                        metadata.artist = tag.artist().unwrap_or("Unknown Artist").to_string();
                        metadata.album = tag.album().unwrap_or("Unknown Album").to_string();
                        
                        // Get album art
                        if let Some(picture) = tag.pictures().next() {
                            metadata.album_art = Some(picture.data.clone());
                        }
                    }
                },
                "flac" => {
                    if let Ok(tag) = FlacTag::read_from_path(path) {
                        if let Some(vorbis) = tag.vorbis_comments() {
                            if let Some(title) = vorbis.title() {
                                metadata.title = title[0].to_string();
                            }
                            if let Some(artist) = vorbis.artist() {
                                metadata.artist = artist[0].to_string();
                            }
                            if let Some(album) = vorbis.album() {
                                metadata.album = album[0].to_string();
                            }
                        }
                        
                        // Get album art
                        if let Some(picture) = tag.pictures().next() {
                            metadata.album_art = Some(picture.data.clone());
                        }
                    }
                },
                _ => {}
            }
        }

        Ok(metadata)
    }
} 