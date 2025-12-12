#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Image {
    id: u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageFormat {
    RGBA8SRGB,
    RGB8SRGB,
    D32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageUsage {
    DepthBuffer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImageView {
    id: u32
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImageError {}

impl Image {
    pub fn new(id: u32) -> Image {
        Image { id }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}

impl ImageView {
    pub fn new(id: u32) -> ImageView {
        ImageView { id }
    }

    pub fn id(&self) -> u32 {
        self.id
    }
}
