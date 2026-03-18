use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

// Single Image Object
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, ToSchema)]
pub struct ImageObject {
    // Essential
    pub url: String,

    // Optional
    pub svg_content: Option<String>,
    pub width: Option<u32>,
    pub height: Option<u32>,
    pub alt: Option<super::Localized<String>>,
    pub size_in_kb: Option<u32>,
    pub mime_type: Option<String>,
    pub blurhash: Option<String>,
}

// Image with Index (for carousel etc)
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, ToSchema)]
pub struct IndexedImage {
    pub index: u32,

    #[serde(flatten)]
    pub image: ImageObject,
}

// image with all possible orientations
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, ToSchema)]
pub struct OrientedImage {
    // Essential
    pub horizontal: super::Localized<ImageObject>,
    pub vertical: super::Localized<ImageObject>,

    // Optional
    pub square: Option<super::Localized<ImageObject>>,
    pub ultrawide: Option<super::Localized<ImageObject>>,
}

// indexed image with all possible orientations
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, ToSchema)]
pub struct OrientedImageArray {
    // Essential
    pub horizontal: Vec<IndexedImage>,
    pub vertical: Vec<IndexedImage>,

    // Optional
    pub square: Option<Vec<IndexedImage>>,
    pub ultrawide: Option<Vec<IndexedImage>>,
}

/// All game images - generalized
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq, ToSchema)]
pub struct GameImages {
    // Essential
    pub hero: OrientedImage,
    pub carousel: OrientedImageArray,

    // Optional
    pub thumbnail: Option<OrientedImage>,
    pub icon: Option<ImageObject>, // usually just one, no orientation
    pub logo: Option<ImageObject>,
    pub screenshots: Option<OrientedImageArray>,
}
