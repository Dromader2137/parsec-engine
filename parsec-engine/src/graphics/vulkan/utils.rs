use crate::math::{ivec::Vec2i, uvec::{Vec2u, Vec3u}};

pub type VulkanExtent2D = ash::vk::Extent2D;
pub type VulkanExtent3D = ash::vk::Extent3D;

impl From<Vec2u> for VulkanExtent2D {
    fn from(value: Vec2u) -> Self {
        VulkanExtent2D {
            width: value.x,
            height: value.y,
        }
    }
}

impl From<Vec3u> for VulkanExtent3D {
    fn from(value: Vec3u) -> Self {
        VulkanExtent3D {
            width: value.x,
            height: value.y,
            depth: value.z,
        }
    }
}

pub type VulkanRect2D = ash::vk::Rect2D;

impl From<Vec2u> for VulkanRect2D {
    fn from(value: Vec2u) -> Self {
        VulkanRect2D {
            offset: Vec2i::ZERO.into(),
            extent: value.into()
        }
    }
}

pub type VulkanOffset2D = ash::vk::Offset2D;

impl From<Vec2i> for VulkanOffset2D {
    fn from(value: Vec2i) -> Self {
        VulkanOffset2D {
            x: value.x,
            y: value.y
        }
    }
}

pub type VulkanResult = ash::vk::Result;
