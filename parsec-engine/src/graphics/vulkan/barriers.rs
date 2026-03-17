use crate::graphics::vulkan::{
    access::VulkanAccess,
    buffer::VulkanBuffer,
    image::{VulkanImage, VulkanImageLayout},
};

pub struct VulkanMemoryBarrier<'a> {
    src_access: &'a [VulkanAccess],
    dst_access: &'a [VulkanAccess],
}

impl<'a> VulkanMemoryBarrier<'a> {
    pub fn new(
        src_access: &'a [VulkanAccess],
        dst_access: &'a [VulkanAccess],
    ) -> Self {
        Self {
            src_access,
            dst_access,
        }
    }

    pub fn raw_memory_barrier(&self) -> ash::vk::MemoryBarrier<'_> {
        ash::vk::MemoryBarrier {
            src_access_mask: VulkanAccess::raw_combined_access_flag(
                self.src_access,
            ),
            dst_access_mask: VulkanAccess::raw_combined_access_flag(
                self.dst_access,
            ),
            ..Default::default()
        }
    }
}

pub struct VulkanBufferMemoryBarrier<'buffer, 'a> {
    src_access: &'a [VulkanAccess],
    dst_access: &'a [VulkanAccess],
    buffer: &'buffer VulkanBuffer,
}

impl<'buffer, 'a> VulkanBufferMemoryBarrier<'buffer, 'a> {
    pub fn new(
        src_access: &'a [VulkanAccess],
        dst_access: &'a [VulkanAccess],
        buffer: &'buffer VulkanBuffer,
    ) -> Self {
        Self {
            src_access,
            dst_access,
            buffer,
        }
    }

    pub fn raw_buffer_memory_barrier(
        &self,
    ) -> ash::vk::BufferMemoryBarrier<'_> {
        ash::vk::BufferMemoryBarrier {
            src_access_mask: VulkanAccess::raw_combined_access_flag(
                self.src_access,
            ),
            dst_access_mask: VulkanAccess::raw_combined_access_flag(
                self.dst_access,
            ),
            buffer: *self.buffer.get_buffer_raw(),
            ..Default::default()
        }
    }
}

pub struct VulkanImageMemoryBarrier<'image, 'a> {
    src_access: &'a [VulkanAccess],
    dst_access: &'a [VulkanAccess],
    src_layout: VulkanImageLayout,
    dst_layout: VulkanImageLayout,
    image: &'image dyn VulkanImage,
}

impl<'image, 'a> VulkanImageMemoryBarrier<'image, 'a> {
    pub fn new(
        src_access: &'a [VulkanAccess],
        dst_access: &'a [VulkanAccess],
        src_layout: VulkanImageLayout,
        dst_layout: VulkanImageLayout,
        image: &'image dyn VulkanImage,
    ) -> Self {
        Self {
            src_access,
            dst_access,
            src_layout,
            dst_layout,
            image,
        }
    }

    pub fn raw_image_memory_barrier(&self) -> ash::vk::ImageMemoryBarrier<'_> {
        ash::vk::ImageMemoryBarrier {
            subresource_range: ash::vk::ImageSubresourceRange {
                aspect_mask: self.image.aspect().raw_image_aspect(),
                level_count: 1,
                layer_count: 1,
                ..Default::default()
            },
            src_access_mask: VulkanAccess::raw_combined_access_flag(
                self.src_access,
            ),
            dst_access_mask: VulkanAccess::raw_combined_access_flag(
                self.dst_access,
            ),
            old_layout: self.src_layout.raw_image_layout(),
            new_layout: self.dst_layout.raw_image_layout(),
            image: *self.image.raw_image(),
            ..Default::default()
        }
    }
}
