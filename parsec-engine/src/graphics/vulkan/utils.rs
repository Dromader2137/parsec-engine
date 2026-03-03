use crate::math::{ivec::Vec2i, uvec::Vec2u};

pub fn raw_extent_2d(dimensions: Vec2u) -> ash::vk::Extent2D {
    ash::vk::Extent2D {
        width: dimensions.x,
        height: dimensions.y
    }
}

pub fn raw_offset_2d(offset: Vec2i) -> ash::vk::Offset2D {
    ash::vk::Offset2D {
        x: offset.x,
        y: offset.y
    }
}

pub fn raw_rect_2d(dimensions: Vec2u, offset: Vec2i) -> ash::vk::Rect2D {
    ash::vk::Rect2D {
        offset: raw_offset_2d(offset),
        extent: raw_extent_2d(dimensions)
    }
}
