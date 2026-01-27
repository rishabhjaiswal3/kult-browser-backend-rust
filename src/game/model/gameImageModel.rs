use serde::{Deserialize, Serialize};

// Single Image Object
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ImageObject {
    // Essential
    pub url: String,

    // Optional
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub alt: Option<String>,
    pub size_in_kb: Option<u32>,
    pub mime_type: Option<String>,
    pub blurhash: Option<String>
}

// Image with Index (for carousel etc)
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IndexedImage {
    pub index: u32,

    #[serde(flatten)]
    pub image: ImageObject
}

// image with all possible orientations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrientedImage {
    // Essential
    pub horizontal: ImageObject,
    pub vertical: ImageObject,

    // Optional
    pub square: Option<ImageObject>,
    pub ultrawide: Option<ImageObject>
}

// indexed image with all possible orientations
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct OrientedImageArray {
    // Essential
    pub horizontal: Vec<IndexedImage>,
    pub vertical: Vec<IndexedImage>,

    // Optional
    pub square: Option<Vec<IndexedImage>>,
    pub ultrawide: Option<Vec<IndexedImage>>
}

/// All game images - generalized
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameImages {
    // Essential
    pub hero: OrientedImage,
    pub carousel: OrientedImageArray,

    // Optional
    pub thumbnail: Option<OrientedImage>,
    pub icon: Option<ImageObject>,  // usually just one, no orientation
    pub logo: Option<ImageObject>,
    pub screenshots: Option<OrientedImageArray>,
}

