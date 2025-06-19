use mime::Mime;
use std::path::Path;

/// Detect MIME type from file extension
pub fn guess_mime_type<P: AsRef<Path>>(path: P) -> String {
    let path = path.as_ref();
    
    match path.extension().and_then(|s| s.to_str()) {
        Some("txt") => "text/plain",
        Some("html") | Some("htm") => "text/html",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("xml") => "application/xml",
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("gif") => "image/gif",
        Some("svg") => "image/svg+xml",
        Some("pdf") => "application/pdf",
        Some("zip") => "application/zip",
        Some("tar") => "application/x-tar",
        Some("gz") => "application/gzip",
        Some("mp3") => "audio/mpeg",
        Some("mp4") => "video/mp4",
        Some("avi") => "video/x-msvideo",
        Some("mov") => "video/quicktime",
        Some("wav") => "audio/wav",
        Some("doc") => "application/msword",
        Some("docx") => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        Some("xls") => "application/vnd.ms-excel",
        Some("xlsx") => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        Some("ppt") => "application/vnd.ms-powerpoint",
        Some("pptx") => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        _ => "application/octet-stream",
    }.to_string()
}

/// Check if MIME type is text-based
pub fn is_text_mime_type(mime_type: &str) -> bool {
    mime_type.starts_with("text/") || 
    matches!(mime_type, 
        "application/json" | 
        "application/xml" | 
        "application/javascript" |
        "application/x-javascript"
    )
}

/// Check if MIME type is an image
pub fn is_image_mime_type(mime_type: &str) -> bool {
    mime_type.starts_with("image/")
}

/// Check if MIME type is a video
pub fn is_video_mime_type(mime_type: &str) -> bool {
    mime_type.starts_with("video/")
}

/// Check if MIME type is audio
pub fn is_audio_mime_type(mime_type: &str) -> bool {
    mime_type.starts_with("audio/")
}

/// Parse MIME type string into mime::Mime
pub fn parse_mime_type(mime_str: &str) -> Result<Mime, mime::FromStrError> {
    mime_str.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_guess_mime_type() {
        assert_eq!(guess_mime_type("test.txt"), "text/plain");
        assert_eq!(guess_mime_type("index.html"), "text/html");
        assert_eq!(guess_mime_type("data.json"), "application/json");
        assert_eq!(guess_mime_type("image.png"), "image/png");
        assert_eq!(guess_mime_type("unknown.xyz"), "application/octet-stream");
    }

    #[test]
    fn test_is_text_mime_type() {
        assert!(is_text_mime_type("text/plain"));
        assert!(is_text_mime_type("text/html"));
        assert!(is_text_mime_type("application/json"));
        assert!(!is_text_mime_type("image/png"));
        assert!(!is_text_mime_type("application/octet-stream"));
    }

    #[test]
    fn test_is_image_mime_type() {
        assert!(is_image_mime_type("image/png"));
        assert!(is_image_mime_type("image/jpeg"));
        assert!(!is_image_mime_type("text/plain"));
        assert!(!is_image_mime_type("video/mp4"));
    }

    #[test]
    fn test_is_video_mime_type() {
        assert!(is_video_mime_type("video/mp4"));
        assert!(is_video_mime_type("video/quicktime"));
        assert!(!is_video_mime_type("audio/mp3"));
        assert!(!is_video_mime_type("image/png"));
    }

    #[test]
    fn test_is_audio_mime_type() {
        assert!(is_audio_mime_type("audio/mp3"));
        assert!(is_audio_mime_type("audio/wav"));
        assert!(!is_audio_mime_type("video/mp4"));
        assert!(!is_audio_mime_type("text/plain"));
    }
}