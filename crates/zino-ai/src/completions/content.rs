//! Content types for AI messages.
//!
//! This module provides support for both simple text content and complex
//! multimodal content including text, images, and other media types.

use serde::{Deserialize, Serialize};

/// Content types for AI messages.
///
/// `Content` supports both simple text messages and complex multimodal content
/// including text, images, and other media types.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum Content {
    /// Simple text content as a string.
    Text(String),
    /// Multimodal content with multiple parts (text, images, etc.).
    Parts(Vec<ContentPart>),
}

/// Individual content parts within multimodal messages.
///
/// `ContentPart` represents a single piece of content that can be text or media.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContentPart {
    /// Text content part.
    Text {
        /// The text content.
        text: String,
    },
    /// Image content part with URL.
    ImageUrl {
        /// The image URL information.
        image_url: ImageUrl,
    },
    /// Video content part with URL.
    VideoUrl {
        /// The video URL information.
        video_url: VideoUrl,
    },
    /// File content part with URL.
    FileUrl {
        /// The file URL information.
        file_url: FileUrl,
    },
    /// Audio input content part.
    InputAudio {
        /// The audio input information.
        input_audio: AudioInput,
    },
}

/// Image URL information for content parts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageUrl {
    /// The URL of the image.
    pub url: String,
}

/// Video URL information for content parts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VideoUrl {
    /// The URL of the video.
    pub url: String,
}

/// File URL information for content parts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FileUrl {
    /// The URL of the file.
    pub url: String,
}

/// Audio input information for content parts.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AudioInput {
    /// The base64 encoded audio data.
    pub data: String,
    /// The audio format (e.g., "wav", "mp3", "m4a").
    pub format: String,
}

impl Content {
    /// Creates a new text content.
    ///
    /// # Arguments
    /// * `text` - The text content
    ///
    /// # Returns
    /// * `Content::Text` - The text content
    pub fn text(text: String) -> Self {
        Content::Text(text)
    }

    /// Creates a new multimodal content with multiple parts.
    ///
    /// # Arguments
    /// * `parts` - Vector of content parts
    ///
    /// # Returns
    /// * `Content::Parts` - The multimodal content
    pub fn parts(parts: Vec<ContentPart>) -> Self {
        Content::Parts(parts)
    }

    /// Creates a multimodal content with text and images.
    ///
    /// # Arguments
    /// * `text` - The text content
    /// * `image_urls` - Vector of image URLs
    ///
    /// # Returns
    /// * `Content::Parts` - The multimodal content
    pub fn text_with_images(text: String, image_urls: Vec<String>) -> Self {
        let mut parts = vec![ContentPart::Text { text }];
        for url in image_urls {
            parts.push(ContentPart::ImageUrl {
                image_url: ImageUrl { url },
            });
        }
        Content::Parts(parts)
    }

    /// Creates a multimodal content with text and videos.
    ///
    /// # Arguments
    /// * `text` - The text content
    /// * `video_urls` - Vector of video URLs
    ///
    /// # Returns
    /// * `Content::Parts` - The multimodal content
    pub fn text_with_videos(text: String, video_urls: Vec<String>) -> Self {
        let mut parts = vec![ContentPart::Text { text }];
        for url in video_urls {
            parts.push(ContentPart::VideoUrl {
                video_url: VideoUrl { url },
            });
        }
        Content::Parts(parts)
    }

    /// Creates a multimodal content with text and files.
    ///
    /// # Arguments
    /// * `text` - The text content
    /// * `file_urls` - Vector of file URLs
    ///
    /// # Returns
    /// * `Content::Parts` - The multimodal content
    pub fn text_with_files(text: String, file_urls: Vec<String>) -> Self {
        let mut parts = vec![ContentPart::Text { text }];
        for url in file_urls {
            parts.push(ContentPart::FileUrl {
                file_url: FileUrl { url },
            });
        }
        Content::Parts(parts)
    }

    /// Creates a multimodal content with text and multiple audio inputs.
    ///
    /// # Arguments
    /// * `text` - The text content
    /// * `audios` - Vector of (data, format) tuples for audio inputs
    ///
    /// # Returns
    /// * `Content::Parts` - The multimodal content
    pub fn text_with_audios(text: String, audios: Vec<(String, String)>) -> Self {
        let mut parts = vec![ContentPart::Text { text }];
        for (data, format) in audios {
            parts.push(ContentPart::InputAudio {
                input_audio: AudioInput { data, format },
            });
        }
        Content::Parts(parts)
    }

    /// Gets the text content if this is a text content.
    ///
    /// # Returns
    /// * `Some(&String)` - If this is text content
    /// * `None` - If this is not text content
    pub fn as_text(&self) -> Option<&String> {
        match self {
            Content::Text(text) => Some(text),
            Content::Parts(_) => None,
        }
    }

    /// Gets the content parts if this is multimodal content.
    ///
    /// # Returns
    /// * `Some(&Vec<ContentPart>)` - If this is multimodal content
    /// * `None` - If this is not multimodal content
    pub fn as_parts(&self) -> Option<&Vec<ContentPart>> {
        match self {
            Content::Text(_) => None,
            Content::Parts(parts) => Some(parts),
        }
    }

    /// Converts the content to a string representation.
    ///
    /// For text content, returns the text directly.
    /// For multimodal content, shows all parts including images with detailed format.
    ///
    /// # Returns
    /// * `String` - The string representation of the content
    pub fn as_string(&self) -> String {
        match self {
            Content::Text(text) => text.clone(),
            Content::Parts(parts) => {
                if parts.len() == 1 {
                    // Single part, show it directly
                    match &parts[0] {
                        ContentPart::Text { text } => text.clone(),
                        ContentPart::ImageUrl { image_url } => {
                            format!("[Image: {}]", image_url.url)
                        }
                        ContentPart::VideoUrl { video_url } => {
                            format!("[Video: {}]", video_url.url)
                        }
                        ContentPart::FileUrl { file_url } => format!("[File: {}]", file_url.url),
                        ContentPart::InputAudio { input_audio } => {
                            format!("[Audio: {} format]", input_audio.format)
                        }
                    }
                } else {
                    // Multiple parts, show as structured format
                    let parts_str = parts
                        .iter()
                        .enumerate()
                        .map(|(i, part)| match part {
                            ContentPart::Text { text } => {
                                format!("  {}. Text: \"{}\"", i + 1, text)
                            }
                            ContentPart::ImageUrl { image_url } => {
                                format!("  {}. Image: {}", i + 1, image_url.url)
                            }
                            ContentPart::VideoUrl { video_url } => {
                                format!("  {}. Video: {}", i + 1, video_url.url)
                            }
                            ContentPart::FileUrl { file_url } => {
                                format!("  {}. File: {}", i + 1, file_url.url)
                            }
                            ContentPart::InputAudio { input_audio } => {
                                format!("  {}. Audio: {} format", i + 1, input_audio.format)
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n");
                    format!("Multimodal Content:\n{}", parts_str)
                }
            }
        }
    }
}

impl std::fmt::Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

impl ContentPart {
    /// Creates a new text content part.
    ///
    /// # Arguments
    /// * `text` - The text content
    ///
    /// # Returns
    /// * `ContentPart::Text` - The text content part
    pub fn text(text: String) -> Self {
        ContentPart::Text { text }
    }

    /// Creates a new image content part.
    ///
    /// # Arguments
    /// * `url` - The image URL
    ///
    /// # Returns
    /// * `ContentPart::ImageUrl` - The image content part
    pub fn image_url(url: String) -> Self {
        ContentPart::ImageUrl {
            image_url: ImageUrl { url },
        }
    }

    /// Creates a new video content part.
    ///
    /// # Arguments
    /// * `url` - The video URL
    ///
    /// # Returns
    /// * `ContentPart::VideoUrl` - The video content part
    pub fn video_url(url: String) -> Self {
        ContentPart::VideoUrl {
            video_url: VideoUrl { url },
        }
    }

    /// Creates a new file content part.
    ///
    /// # Arguments
    /// * `url` - The file URL
    ///
    /// # Returns
    /// * `ContentPart::FileUrl` - The file content part
    pub fn file_url(url: String) -> Self {
        ContentPart::FileUrl {
            file_url: FileUrl { url },
        }
    }

    /// Creates a new audio input content part.
    ///
    /// # Arguments
    /// * `data` - The base64 encoded audio data
    /// * `format` - The audio format (e.g., "wav", "mp3", "m4a")
    ///
    /// # Returns
    /// * `ContentPart::InputAudio` - The audio input content part
    pub fn input_audio(data: String, format: String) -> Self {
        ContentPart::InputAudio {
            input_audio: AudioInput { data, format },
        }
    }
}
