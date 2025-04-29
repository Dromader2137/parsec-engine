use crate::graphics::{renderer::Renderer, window::WindowWrapper};

use super::{command_buffer::CommandBuffer, context::{VulkanContext, VulkanError}, fence::Fence, framebuffer::Framebuffer, renderpass::Renderpass, semaphore::Semaphore};

pub struct VulkanRenderer {
    renderpass: Renderpass,
    framebuffers: Vec<Framebuffer>,
    command_buffer: CommandBuffer,
    command_buffer_reuse_fence: Fence,
    rendering_semaphore: Semaphore,
    present_semaphore: Semaphore,
}

impl VulkanRenderer {
    pub fn new(context: &VulkanContext, window: &WindowWrapper) -> Result<VulkanRenderer, VulkanError> {
        let renderpass = Renderpass::new(&context.surface, &context.device)?;
        let framebuffers = {
            let mut out = Vec::new();
            for image_view in context.swapchain_image_views.iter() {
                out.push(Framebuffer::new(&context.surface, &context.device, image_view, &renderpass, window)?);
            }
            out
        };
        let command_buffer = CommandBuffer::new(&context.device, &context.command_pool)?;
        command_buffer.begin(&context.device)?;
        command_buffer.end(&context.device)?;
        let command_buffer_reuse_fence = Fence::new(&context.device, true)?;
        let rendering_semaphore = Semaphore::new(&context.device)?;
        let present_semaphore = Semaphore::new(&context.device)?;

        Ok(VulkanRenderer { renderpass, framebuffers, command_buffer, command_buffer_reuse_fence, rendering_semaphore, present_semaphore })
    }
}

impl Renderer for VulkanRenderer {
    fn handle_resize(&mut self) -> Result<(), crate::error::EngineError> {
        Ok(())
    }

    fn render(
        &mut self,
        vulkan_context: &VulkanContext,
        window: &WindowWrapper,
    ) -> Result<(), crate::error::EngineError> {
        let device = vulkan_context.device.get_device_raw();
        let command_buffer = *self.command_buffer.get_command_buffer_raw();
        let command_buffer_reuse_fence = *self.command_buffer_reuse_fence.get_fence_raw();
        let submit_queue = *vulkan_context.graphics_queue.get_queue_raw();
        let present_semaphores = &[*self.present_semaphore.get_semaphore_raw()];
        let render_semaphores = &[*self.rendering_semaphore.get_semaphore_raw()];
        let wait_mask = &[ash::vk::PipelineStageFlags::COLOR_ATTACHMENT_OUTPUT];
        let swapchains = [*vulkan_context.swapchain.get_swapchain_raw()];

        unsafe {
            let (present_index, _) = vulkan_context
                .swapchain
                .get_swapchain_loader_raw()
                .acquire_next_image(
                    *vulkan_context.swapchain.get_swapchain_raw(),
                    u64::MAX,
                    *self.present_semaphore.get_semaphore_raw(),
                    ash::vk::Fence::null(),
                )
                .unwrap();

            device
                .wait_for_fences(&[command_buffer_reuse_fence], true, u64::MAX)
                .expect("Wait for fence failed.");

            device
                .reset_fences(&[command_buffer_reuse_fence])
                .expect("Reset fences failed.");

            device
                .reset_command_buffer(
                    command_buffer,
                    ash::vk::CommandBufferResetFlags::RELEASE_RESOURCES,
                )
                .expect("Reset command buffer failed.");

            let command_buffer_begin_info = ash::vk::CommandBufferBeginInfo::default()
                .flags(ash::vk::CommandBufferUsageFlags::SIMULTANEOUS_USE);

            device
                .begin_command_buffer(command_buffer, &command_buffer_begin_info)
                .expect("Begin commandbuffer");
            device
                .end_command_buffer(command_buffer)
                .expect("End commandbuffer");

            let command_buffers = vec![command_buffer];

            let submit_info = ash::vk::SubmitInfo::default()
                .wait_semaphores(present_semaphores)
                .wait_dst_stage_mask(wait_mask)
                .command_buffers(&command_buffers)
                .signal_semaphores(render_semaphores);

            device
                .queue_submit(submit_queue, &[submit_info], command_buffer_reuse_fence)
                .expect("queue submit failed.");
                        
            let image_indices = [present_index];
            let present_info = ash::vk::PresentInfoKHR::default()
                .wait_semaphores(render_semaphores)
                .swapchains(&swapchains)
                .image_indices(&image_indices);

            vulkan_context.swapchain
                .get_swapchain_loader_raw()
                .queue_present(submit_queue, &present_info)
                .unwrap();
        }

        Ok(())
    }
}
