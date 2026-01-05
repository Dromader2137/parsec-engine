//! Module responsible for interaction with the Vulkan API. (Incomplete, undocumented and subject
//! to change).

use std::collections::HashMap;

use crate::{
    graphics::{
        backend::{BackendInitError, GraphicsBackend},
        buffer::{Buffer, BufferError, BufferUsage},
        command_list::{CommandList, CommandListError},
        fence::{Fence, FenceError},
        framebuffer::{Framebuffer, FramebufferError},
        image::{Image, ImageError, ImageFlag, ImageFormat, ImageView},
        pipeline::{
            Pipeline, PipelineBinding, PipelineBindingLayout, PipelineError,
            PipelineOptions, PipelineStage, PipelineSubbindingLayout,
        },
        renderer::DefaultVertex,
        renderpass::{Renderpass, RenderpassAttachment, RenderpassError},
        sampler::{Sampler, SamplerError},
        semaphore::{Semaphore, SemaphoreError},
        shader::{Shader, ShaderError, ShaderType},
        swapchain::{Swapchain, SwapchainError},
        vulkan::{
            buffer::{VulkanBuffer, VulkanBufferUsage},
            command_buffer::{
                VulkanAccess, VulkanCommandBuffer, VulkanCommandPool,
                VulkanImageMemoryBarrier, VulkanPipelineStage,
            },
            descriptor_set::{
                DescriptorType, VulkanDescriptorPool, VulkanDescriptorPoolSize,
                VulkanDescriptorSet, VulkanDescriptorSetBinding,
                VulkanDescriptorSetLayout,
            },
            device::VulkanDevice,
            fence::VulkanFence,
            framebuffer::VulkanFramebuffer,
            graphics_pipeline::VulkanGraphicsPipeline,
            image::{
                VulkanImage, VulkanImageAspectFlags, VulkanImageInfo,
                VulkanImageLayout, VulkanImageUsage, VulkanImageView,
                VulkanOwnedImage, VulkanSwapchainImage,
            },
            instance::VulkanInstance,
            physical_device::VulkanPhysicalDevice,
            queue::VulkanQueue,
            renderpass::VulkanRenderpass,
            sampler::VulkanSampler,
            semaphore::VulkanSemaphore,
            shader::VulkanShaderModule,
            surface::{VulkanInitialSurface, VulkanSurface},
            swapchain::{VulkanSwapchain, VulkanSwapchainError},
        },
        window::Window,
    },
    math::uvec::Vec2u,
};

mod allocation;
mod allocator;
mod buffer;
mod command_buffer;
mod descriptor_set;
mod device;
mod fence;
mod format_size;
mod framebuffer;
mod graphics_pipeline;
mod image;
mod instance;
mod physical_device;
mod queue;
mod renderpass;
mod sampler;
mod semaphore;
pub mod shader;
mod surface;
mod swapchain;
mod utils;

#[allow(unused)]
pub struct VulkanBackend {
    instance: VulkanInstance,
    physical_device: VulkanPhysicalDevice,
    surface: VulkanSurface,
    device: VulkanDevice,
    command_pool: VulkanCommandPool,
    present_queue: VulkanQueue,
    descriptor_pool: VulkanDescriptorPool,
    swapchains: HashMap<u32, VulkanSwapchain>,
    swapchain_images: HashMap<u32, VulkanSwapchainImage>,
    owned_images: HashMap<u32, VulkanOwnedImage>,
    image_views: HashMap<u32, VulkanImageView>,
    samplers: HashMap<u32, VulkanSampler>,
    framebuffers: HashMap<u32, VulkanFramebuffer>,
    fences: HashMap<u32, VulkanFence>,
    semaphores: HashMap<u32, VulkanSemaphore>,
    command_buffers: HashMap<u32, VulkanCommandBuffer>,
    buffers: HashMap<u32, VulkanBuffer>,
    shaders: HashMap<u32, VulkanShaderModule>,
    pipelines: HashMap<u32, VulkanGraphicsPipeline>,
    renderpasses: HashMap<u32, VulkanRenderpass>,
    descriptor_sets: HashMap<u32, VulkanDescriptorSet>,
    descriptor_set_layouts: HashMap<u32, VulkanDescriptorSetLayout>,
}

impl GraphicsBackend for VulkanBackend {
    fn init(window: &Window) -> Result<Self, BackendInitError> {
        let instance = VulkanInstance::new(&window)
            .map_err(|err| BackendInitError::InitError(err.into()))?;
        let initial_surface = VulkanInitialSurface::new(&instance, &window)
            .map_err(|err| BackendInitError::InitError(err.into()))?;
        let physical_device =
            VulkanPhysicalDevice::new(&instance, &initial_surface)
                .map_err(|err| BackendInitError::InitError(err.into()))?;
        let surface = VulkanSurface::from_initial_surface(
            initial_surface,
            &physical_device,
        )
        .map_err(|err| BackendInitError::InitError(err.into()))?;
        let device = VulkanDevice::new(&instance, &physical_device, &surface)
            .map_err(|err| BackendInitError::InitError(err.into()))?;
        let command_pool = VulkanCommandPool::new(&physical_device, &device)
            .map_err(|err| BackendInitError::InitError(err.into()))?;
        let present_queue = VulkanQueue::present(
            &device,
            physical_device.get_queue_family_index(),
        );
        let descriptor_pool = VulkanDescriptorPool::new(&device, 128, &[
            VulkanDescriptorPoolSize::new(128, DescriptorType::UNIFORM_BUFFER),
            VulkanDescriptorPoolSize::new(
                16,
                DescriptorType::COMBINED_IMAGE_SAMPLER,
            ),
        ])
        .map_err(|err| BackendInitError::InitError(err.into()))?;
        Ok(VulkanBackend {
            instance,
            physical_device,
            surface,
            device,
            command_pool,
            present_queue,
            descriptor_pool,
            swapchains: HashMap::new(),
            swapchain_images: HashMap::new(),
            owned_images: HashMap::new(),
            image_views: HashMap::new(),
            samplers: HashMap::new(),
            framebuffers: HashMap::new(),
            fences: HashMap::new(),
            semaphores: HashMap::new(),
            command_buffers: HashMap::new(),
            buffers: HashMap::new(),
            shaders: HashMap::new(),
            pipelines: HashMap::new(),
            renderpasses: HashMap::new(),
            descriptor_sets: HashMap::new(),
            descriptor_set_layouts: HashMap::new(),
        })
    }

    fn wait_idle(&self) { self.device.wait_idle().unwrap(); }

    fn get_surface_format(&self) -> ImageFormat { self.surface.format().into() }

    fn create_buffer(
        &mut self,
        data: &[impl Copy],
        buffer_usage: &[BufferUsage],
    ) -> Result<Buffer, BufferError> {
        let mut usage = VulkanBufferUsage::empty();
        buffer_usage.iter().for_each(|x| usage |= (*x).into());
        let buffer = VulkanBuffer::from_vec(&self.device, data, usage)
            .map_err(|err| BufferError::BufferCreationError(err.into()))?;
        let buffer_id = buffer.id();
        self.buffers.insert(buffer_id, buffer);
        Ok(Buffer::new(buffer_id))
    }

    fn update_buffer(
        &mut self,
        buffer: Buffer,
        data: &[impl Copy],
    ) -> Result<(), BufferError> {
        let buffer_id = buffer.id();
        let buffer = self
            .buffers
            .get(&buffer_id)
            .ok_or(BufferError::BufferNotFound)?;
        buffer
            .update(&self.device, data)
            .map_err(|err| BufferError::BufferUpdateError(err.into()))?;
        Ok(())
    }

    fn delete_buffer(&mut self, buffer: Buffer) -> Result<(), BufferError> {
        let buffer = self
            .buffers
            .remove(&buffer.id())
            .ok_or(BufferError::BufferNotFound)?;
        buffer
            .delete_buffer(&self.device)
            .map_err(|err| BufferError::BufferDeletionError(err.into()))
    }

    fn create_shader(
        &mut self,
        code: &[u32],
        shader_type: ShaderType,
    ) -> Result<Shader, ShaderError> {
        let shader =
            VulkanShaderModule::new(&self.device, code, shader_type)
                .map_err(|err| ShaderError::ShaderCreationError(err.into()))?;
        let shader_id = shader.id();
        self.shaders.insert(shader_id, shader);
        Ok(Shader::new(shader_id))
    }

    fn delete_shader(&mut self, shader: Shader) -> Result<(), ShaderError> {
        let shader = self
            .shaders
            .remove(&shader.id())
            .ok_or(ShaderError::ShaderNotFound)?;
        shader
            .delete_shader(&self.device)
            .map_err(|err| ShaderError::ShaderDeletionError(err.into()))
    }

    fn create_renderpass(
        &mut self,
        attachments: &[RenderpassAttachment],
    ) -> Result<Renderpass, RenderpassError> {
        let renderpass = VulkanRenderpass::new(
            &self.surface,
            &self.device,
            &attachments.iter().map(|x| (*x).into()).collect::<Vec<_>>(),
        )
        .map_err(|err| RenderpassError::RenderpassCreationError(err.into()))?;
        let rendrepass_id = renderpass.id();
        self.renderpasses.insert(rendrepass_id, renderpass);
        Ok(Renderpass::new(rendrepass_id))
    }

    fn delete_renderpass(
        &mut self,
        renderpass: Renderpass,
    ) -> Result<(), RenderpassError> {
        let renderpass = self
            .renderpasses
            .remove(&renderpass.id())
            .ok_or(RenderpassError::RenderpassNotFound)?;
        renderpass
            .delete_renderpass(&self.device)
            .map_err(|err| RenderpassError::RenderpassDeletionError(err.into()))
    }

    fn create_pipeline_binding_layout(
        &mut self,
        subbindings: &[PipelineSubbindingLayout],
    ) -> Result<PipelineBindingLayout, PipelineError> {
        let set_bindings = subbindings
            .iter()
            .enumerate()
            .map(|(id, binding)| {
                VulkanDescriptorSetBinding::new(
                    id as u32,
                    binding.binding_type.into(),
                    binding.shader_stage.into(),
                )
            })
            .collect::<Vec<_>>();
        let dsl = VulkanDescriptorSetLayout::new(&self.device, set_bindings)
            .map_err(|err| PipelineError::LayoutCreationError(err.into()))?;
        let dsl_id = dsl.id();
        self.descriptor_set_layouts.insert(dsl_id, dsl);
        Ok(PipelineBindingLayout::new(dsl_id))
    }

    fn create_pipeline(
        &mut self,
        vertex_shader: Shader,
        fragment_shader: Shader,
        renderpass: Renderpass,
        binding_layouts: &[PipelineBindingLayout],
        options: PipelineOptions,
    ) -> Result<Pipeline, PipelineError> {
        let vsm = self
            .shaders
            .get(&vertex_shader.id())
            .ok_or(PipelineError::ShaderNotFound)?;
        let fsm = self
            .shaders
            .get(&fragment_shader.id())
            .ok_or(PipelineError::ShaderNotFound)?;
        let ren = self
            .renderpasses
            .get(&renderpass.id())
            .ok_or(PipelineError::RenderpassNotFound)?;
        let dsl = binding_layouts
            .into_iter()
            .map(|x| {
                self.descriptor_set_layouts
                    .get(&x.id())
                    .ok_or(PipelineError::LayoutNotFound)
                    .map(|o| o.clone())
            })
            .collect::<Result<Vec<_>, _>>()?;
        let pipeline = VulkanGraphicsPipeline::new::<DefaultVertex>(
            &self.device,
            ren,
            vsm,
            fsm,
            &dsl,
            options.into(),
        )
        .map_err(|err| PipelineError::PipelineCreationError(err.into()))?;
        let pipeline_id = pipeline.id();
        self.pipelines.insert(pipeline_id, pipeline);
        Ok(Pipeline::new(pipeline_id))
    }

    fn create_pipeline_binding(
        &mut self,
        pipeline_layout: PipelineBindingLayout,
    ) -> Result<PipelineBinding, PipelineError> {
        let dsl = self
            .descriptor_set_layouts
            .get(&pipeline_layout.id())
            .ok_or(PipelineError::BindingLayoutNotFound)?;
        let ds =
            VulkanDescriptorSet::new(&self.device, dsl, &self.descriptor_pool)
                .map_err(|err| {
                    PipelineError::BindingCreationError(err.into())
                })?;
        let ds_id = ds.id();
        self.descriptor_sets.insert(ds_id, ds);
        Ok(PipelineBinding::new(ds_id))
    }

    fn bind_buffer(
        &mut self,
        pipeline_binding: PipelineBinding,
        buffer: Buffer,
        index: u32,
    ) -> Result<(), BufferError> {
        let ds = self.descriptor_sets.get(&pipeline_binding.id()).unwrap();
        let dsl = self
            .descriptor_set_layouts
            .get(&ds.descriptor_layout_id())
            .ok_or(BufferError::PipelineBindingNotFound)?;
        let buf = self
            .buffers
            .get(&buffer.id())
            .ok_or(BufferError::BufferNotFound)?;
        ds.bind_buffer(buf, &self.device, dsl, index)
            .map_err(|err| BufferError::BufferBindError(err.into()))
    }

    fn bind_sampler(
        &mut self,
        pipeline_binding: PipelineBinding,
        sampler: Sampler,
        image_view: ImageView,
        index: u32,
    ) -> Result<(), SamplerError> {
        let ds = self
            .descriptor_sets
            .get(&pipeline_binding.id())
            .ok_or(SamplerError::PipelineBindingNotFound)?;
        let dsl = self
            .descriptor_set_layouts
            .get(&ds.descriptor_layout_id())
            .ok_or(SamplerError::PipelineBindingLayoutNotFound)?;
        let sam = self
            .samplers
            .get(&sampler.id())
            .ok_or(SamplerError::SamplerNotFound)?;
        let imv = self
            .image_views
            .get(&image_view.id())
            .ok_or(SamplerError::ImageViewNowFound)?;
        ds.bind_image_view(imv, sam, &self.device, dsl, index)
            .map_err(|err| SamplerError::SamplerBindError(err.into()))
    }

    fn create_command_list(&mut self) -> Result<CommandList, CommandListError> {
        let comman_buffer =
            VulkanCommandBuffer::new(&self.device, &self.command_pool)
                .map_err(|err| {
                    CommandListError::CommandListCreationError(err.into())
                })?;
        let command_buffer_id = comman_buffer.id();
        self.command_buffers
            .insert(command_buffer_id, comman_buffer);
        Ok(CommandList::new(command_buffer_id))
    }

    fn command_begin(
        &mut self,
        command_list: CommandList,
    ) -> Result<(), CommandListError> {
        let command_buffer = self
            .command_buffers
            .get(&command_list.id())
            .ok_or(CommandListError::CommandListNotFound)?;
        command_buffer
            .begin(&self.device)
            .map_err(|err| CommandListError::CommandListBeginError(err.into()))
    }

    fn command_end(
        &mut self,
        command_list: CommandList,
    ) -> Result<(), CommandListError> {
        let command_buffer = self
            .command_buffers
            .get(&command_list.id())
            .ok_or(CommandListError::CommandListNotFound)?;
        command_buffer
            .end(&self.device)
            .map_err(|err| CommandListError::CommandListEndError(err.into()))
    }

    fn command_reset(
        &mut self,
        command_list: CommandList,
    ) -> Result<(), CommandListError> {
        let command_buffer = self
            .command_buffers
            .get(&command_list.id())
            .ok_or(CommandListError::CommandListNotFound)?;
        command_buffer
            .reset(&self.device)
            .map_err(|err| CommandListError::CommandListResetError(err.into()))
    }

    fn command_begin_renderpass(
        &mut self,
        command_list: CommandList,
        renderpass: Renderpass,
        framebuffer: Framebuffer,
    ) -> Result<(), CommandListError> {
        let command_buffer = self
            .command_buffers
            .get(&command_list.id())
            .ok_or(CommandListError::CommandListNotFound)?;
        let fra = self
            .framebuffers
            .get(&framebuffer.id())
            .ok_or(CommandListError::FramebufferNotFound)?;
        let ren = self
            .renderpasses
            .get(&renderpass.id())
            .ok_or(CommandListError::RenderpassNotFound)?;
        let barriers = fra
            .attached_images_ids()
            .iter()
            .enumerate()
            .filter_map(|(id, image_id)| {
                if ren.depth_attachment_id() != id as u32 {
                    return None;
                }

                let o_image = self
                    .owned_images
                    .get_mut(&image_id)
                    .map(|img| img as &mut (dyn VulkanImage + 'static));
                let s_image = self
                    .swapchain_images
                    .get_mut(&image_id)
                    .map(|img| img as &mut (dyn VulkanImage + 'static));
                Some(o_image
                    .or(s_image)
                    .ok_or(CommandListError::ImageNotFound)
                    .map(|image| {
                        VulkanImageMemoryBarrier::raw_image_barrier(
                            image,
                            VulkanImageLayout::DEPTH_ATTACHMENT_OPTIMAL,
                            VulkanAccess::empty()
                        )
                    }))
            })
            .collect::<Result<Result<Vec<_>, _>, _>>()?
            .map_err(|err| {
                CommandListError::CommandListRenderpassBeginError(err.into())
            })?;
        command_buffer
            .pipeline_barrier(
                &self.device,
                VulkanPipelineStage::ALL_GRAPHICS,
                VulkanPipelineStage::EARLY_FRAGMENT_TESTS,
                &[],
                &[],
                &barriers,
            )
            .map_err(|err| CommandListError::CommandListRenderpassBeginError(err.into()))?;
        command_buffer
            .begin_renderpass(&self.device, fra, ren)
            .map_err(|err| {
                CommandListError::CommandListRenderpassBeginError(err.into())
            })?;
        command_buffer
            .set_viewports(&self.device, fra.dimensions(), ren)
            .map_err(|err| {
                CommandListError::CommandListRenderpassBeginError(err.into())
            })?;
        command_buffer
            .set_scissor(&self.device, fra.dimensions(), ren)
            .map_err(|err| {
                CommandListError::CommandListRenderpassBeginError(err.into())
            })
    }

    fn command_end_renderpass(
        &mut self,
        command_list: CommandList,
        renderpass: Renderpass,
        framebuffer: Framebuffer,
    ) -> Result<(), CommandListError> {
        let command_buffer = self
            .command_buffers
            .get(&command_list.id())
            .ok_or(CommandListError::CommandListNotFound)?;
        let fra = self
            .framebuffers
            .get(&framebuffer.id())
            .ok_or(CommandListError::FramebufferNotFound)?;
        let ren = self
            .renderpasses
            .get(&renderpass.id())
            .ok_or(CommandListError::RenderpassNotFound)?;
        let barriers = fra
            .attached_images_ids()
            .iter()
            .enumerate()
            .filter_map(|(id, image_id)| {
                if ren.depth_attachment_id() != id as u32 {
                    return None;
                }

                let o_image = self
                    .owned_images
                    .get_mut(&image_id)
                    .map(|img| img as &mut (dyn VulkanImage + 'static));
                let s_image = self
                    .swapchain_images
                    .get_mut(&image_id)
                    .map(|img| img as &mut (dyn VulkanImage + 'static));
                Some(o_image
                    .or(s_image)
                    .ok_or(CommandListError::ImageNotFound)
                    .map(|image| {
                        VulkanImageMemoryBarrier::raw_image_barrier(
                            image,
                            VulkanImageLayout::GENERAL,
                            VulkanAccess::empty()
                        )
                    }))
            })
            .collect::<Result<Result<Vec<_>, _>, _>>()?
            .map_err(|err| {
                CommandListError::CommandListRenderpassBeginError(err.into())
            })?;
        command_buffer
            .pipeline_barrier(
                &self.device,
                VulkanPipelineStage::ALL_GRAPHICS,
                VulkanPipelineStage::TOP_OF_PIPE,
                &[],
                &[],
                &barriers,
            )
            .map_err(|err| CommandListError::CommandListRenderpassBeginError(err.into()))?;
        command_buffer.end_renderpass(&self.device).map_err(|err| {
            CommandListError::CommandListRenderpassEndError(err.into())
        })
    }

    fn command_bind_pipeline(
        &mut self,
        command_list: CommandList,
        pipeline: Pipeline,
    ) -> Result<(), CommandListError> {
        let command_buffer = self
            .command_buffers
            .get(&command_list.id())
            .ok_or(CommandListError::CommandListNotFound)?;
        let pip = self
            .pipelines
            .get(&pipeline.id())
            .ok_or(CommandListError::PipelineNotFound)?;
        command_buffer
            .bind_graphics_pipeline(&self.device, pip)
            .map_err(|err| CommandListError::CommandListBindError(err.into()))
    }

    fn command_bind_pipeline_binding(
        &mut self,
        command_list: CommandList,
        pipeline: Pipeline,
        binding: PipelineBinding,
        binding_index: u32,
    ) -> Result<(), CommandListError> {
        let command_buffer = self
            .command_buffers
            .get(&command_list.id())
            .ok_or(CommandListError::CommandListNotFound)?;
        let ds = self
            .descriptor_sets
            .get(&binding.id())
            .ok_or(CommandListError::PipelineLayoutNotFound)?;
        let pip = self
            .pipelines
            .get(&pipeline.id())
            .ok_or(CommandListError::PipelineNotFound)?;
        command_buffer
            .bind_descriptor_set(&self.device, ds, pip, binding_index)
            .map_err(|err| CommandListError::CommandListBindError(err.into()))
    }

    fn command_draw(
        &mut self,
        command_list: CommandList,
        vertex_buffer: Buffer,
        index_buffer: Buffer,
    ) -> Result<(), CommandListError> {
        let command_buffer = self
            .command_buffers
            .get(&command_list.id())
            .ok_or(CommandListError::CommandListNotFound)?;
        let vb = self
            .buffers
            .get(&vertex_buffer.id())
            .ok_or(CommandListError::BufferNotFound)?;
        let ib = self
            .buffers
            .get(&index_buffer.id())
            .ok_or(CommandListError::BufferNotFound)?;
        command_buffer
            .bind_vertex_buffer(&self.device, vb)
            .map_err(|err| {
                CommandListError::CommandListDrawError(err.into())
            })?;
        command_buffer
            .bind_index_buffer(&self.device, ib)
            .map_err(|err| {
                CommandListError::CommandListDrawError(err.into())
            })?;
        command_buffer
            .draw_indexed(&self.device, ib.len, 1, 0, 0, 1)
            .map_err(|err| CommandListError::CommandListDrawError(err.into()))
    }

    fn submit_commands(
        &mut self,
        command_list: CommandList,
        wait_semaphores: &[Semaphore],
        signal_semaphores: &[Semaphore],
        signal_fence: Fence,
    ) -> Result<(), CommandListError> {
        let command_buffer = self
            .command_buffers
            .get(&command_list.id())
            .ok_or(CommandListError::CommandListNotFound)?;
        let ws = wait_semaphores
            .iter()
            .map(|x| {
                self.semaphores
                    .get(&x.id())
                    .ok_or(CommandListError::SemaphoreNotFound)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let ss = signal_semaphores
            .iter()
            .map(|x| {
                self.semaphores
                    .get(&x.id())
                    .ok_or(CommandListError::SemaphoreNotFound)
            })
            .collect::<Result<Vec<_>, _>>()?;
        let fen = self
            .fences
            .get(&signal_fence.id())
            .ok_or(CommandListError::FenceNotFound)?;

        self.present_queue
            .submit(&self.device, &ws, &ss, &[command_buffer], &fen)
            .map_err(|err| CommandListError::CommandListSubmitError(err.into()))
    }

    fn create_swapchain(
        &mut self,
        window: &Window,
        old_swapchain: Option<Swapchain>,
    ) -> Result<(Swapchain, Vec<Image>), SwapchainError> {
        let os = match old_swapchain {
            Some(val) => match self.swapchains.get(&val.id()) {
                Some(val) => Some(val),
                None => return Err(SwapchainError::OldSwapchainNotFound),
            },
            None => None,
        };
        let (swapchain, swapchain_images) = VulkanSwapchain::new(
            &self.instance,
            &self.physical_device,
            window,
            &self.surface,
            &self.device,
            os,
        )
        .map_err(|err| SwapchainError::SwapchainCreationError(err.into()))?;
        let swapchain_id = swapchain.id();
        let mut images = Vec::new();
        self.swapchains.insert(swapchain_id, swapchain);
        for swapchain_image in swapchain_images.iter() {
            let swapchain_image_id = swapchain_image.id();
            self.swapchain_images
                .insert(swapchain_image_id, swapchain_image.clone());
            images.push(Image::new(swapchain_image_id));
        }
        Ok((Swapchain::new(swapchain_id), images))
    }

    fn delete_swapchain(
        &mut self,
        swapchain: Swapchain,
    ) -> Result<(), SwapchainError> {
        self.swapchains
            .remove(&swapchain.id())
            .ok_or(SwapchainError::SwapchainNotFound)?;
        Ok(())
    }

    fn next_image_id(
        &mut self,
        swapchain: Swapchain,
        signal_semaphore: Semaphore,
    ) -> Result<u32, SwapchainError> {
        let ss = self
            .semaphores
            .get(&signal_semaphore.id())
            .ok_or(SwapchainError::SemaphoreNotFound)?;

        self.swapchains
            .get(&swapchain.id())
            .ok_or(SwapchainError::SwapchainNotFound)?
            .acquire_next_image(ss, &VulkanFence::null(&self.device))
            .map_err(|err| match err {
                VulkanSwapchainError::OutOfDate => {
                    SwapchainError::SwapchainOutOfDate
                },
                _ => SwapchainError::Undefined,
            })
            .map(|val| val.0)
    }

    fn present(
        &mut self,
        swapchain: Swapchain,
        wait_semaphores: &[Semaphore],
        present_image_index: u32,
    ) -> Result<(), SwapchainError> {
        let ws = wait_semaphores
            .iter()
            .map(|x| {
                self.semaphores
                    .get(&x.id())
                    .ok_or(SwapchainError::SemaphoreNotFound)
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.swapchains
            .get(&swapchain.id())
            .ok_or(SwapchainError::SwapchainNotFound)?
            .present(&self.present_queue, &ws, present_image_index)
            .map_err(|err| match err {
                VulkanSwapchainError::OutOfDate => {
                    SwapchainError::SwapchainOutOfDate
                },
                _ => SwapchainError::Undefined,
            })
    }

    fn create_image(
        &mut self,
        size: Vec2u,
        format: ImageFormat,
        image_usage: &[ImageFlag],
    ) -> Result<Image, ImageError> {
        let mut usage = VulkanImageUsage::empty();
        let mut aspect = VulkanImageAspectFlags::empty();
        image_usage.iter().for_each(|x| usage |= (*x).into());
        image_usage.iter().for_each(|x| aspect |= (*x).into());
        let image = VulkanOwnedImage::new(&self.device, VulkanImageInfo {
            format: format.into(),
            size: (size.x, size.y),
            usage,
            aspect,
        })
        .map_err(|err| ImageError::ImageCreationError(err.into()))?;
        let image_id = image.id();
        self.owned_images.insert(image_id, image);
        Ok(Image::new(image_id))
    }

    fn delete_image(&mut self, image: Image) -> Result<(), ImageError> {
        let result = self.owned_images.remove(&image.id());
        if let Some(image) = result {
            image
                .delete_image(&self.device)
                .map_err(|err| ImageError::ImageDeletionError(err.into()))
        } else {
            self.swapchain_images
                .remove(&image.id())
                .ok_or(ImageError::SwapchainImageNotFound)
                .map(|_| ())
        }
    }

    fn load_image_from_buffer(
        &mut self,
        buffer: Buffer,
        image: Image,
    ) -> Result<(), ImageError> {
        let buf = self
            .buffers
            .get(&buffer.id())
            .ok_or(ImageError::BufferNotFound)?;
        let img = self
            .owned_images
            .get_mut(&image.id())
            .ok_or(ImageError::ImageNotFound)?;
        let cmd = VulkanCommandBuffer::new(&self.device, &self.command_pool)
            .map_err(|err| ImageError::ImageLoadError(err.into()))?;
        cmd.begin(&self.device)
            .map_err(|err| ImageError::ImageLoadError(err.into()))?;
        cmd.pipeline_barrier(
            &self.device,
            VulkanPipelineStage::BOTTOM_OF_PIPE,
            VulkanPipelineStage::TRANSFER,
            &[],
            &[],
            &[VulkanImageMemoryBarrier::raw_image_barrier(
                img,
                VulkanImageLayout::TRANSFER_DST_OPTIMAL,
                VulkanAccess::TRANSFER_WRITE,
            )
            .map_err(|err| ImageError::ImageLoadError(err.into()))?],
        )
        .map_err(|err| ImageError::ImageLoadError(err.into()))?;
        cmd.copy_buffer_to_image(&self.device, buf, img)
            .map_err(|err| ImageError::ImageLoadError(err.into()))?;
        cmd.pipeline_barrier(
            &self.device,
            VulkanPipelineStage::TRANSFER,
            VulkanPipelineStage::FRAGMENT_SHADER,
            &[],
            &[],
            &[VulkanImageMemoryBarrier::raw_image_barrier(
                img,
                VulkanImageLayout::SHADER_READ_ONLY_OPTIMAL,
                VulkanAccess::SHADER_READ,
            )
            .map_err(|err| ImageError::ImageLoadError(err.into()))?],
        )
        .map_err(|err| ImageError::ImageLoadError(err.into()))?;
        cmd.end(&self.device)
            .map_err(|err| ImageError::ImageLoadError(err.into()))?;
        self.present_queue
            .submit(
                &self.device,
                &[],
                &[],
                &[&cmd],
                &VulkanFence::null(&self.device),
            )
            .map_err(|err| ImageError::ImageLoadError(err.into()))
    }

    fn create_image_view(
        &mut self,
        image: Image,
    ) -> Result<ImageView, ImageError> {
        let oim = match self.owned_images.get(&image.id()) {
            Some(val) => match VulkanImageView::from_image(&self.device, val) {
                Ok(val) => Some(val),
                Err(err) => {
                    return Err(ImageError::ImageViewCreationError(err.into()));
                },
            },

            None => None,
        };
        let sim = match self.swapchain_images.get(&image.id()) {
            Some(val) => match VulkanImageView::from_image(&self.device, val) {
                Ok(val) => Some(val),
                Err(err) => {
                    return Err(ImageError::ImageViewCreationError(err.into()));
                },
            },

            None => None,
        };
        let im = oim.or(sim).ok_or(ImageError::ImageNotFound)?;
        let im_id = im.id();
        self.image_views.insert(im_id, im);
        Ok(ImageView::new(im_id))
    }

    fn delete_image_view(
        &mut self,
        image_view: ImageView,
    ) -> Result<(), ImageError> {
        let image_view = self
            .image_views
            .remove(&image_view.id())
            .ok_or(ImageError::ImageViewNotFound)?;
        image_view
            .delete_image_view(&self.device)
            .map_err(|err| ImageError::ImageDeletionError(err.into()))
    }

    fn create_image_sampler(&mut self) -> Result<Sampler, SamplerError> {
        let sampler = VulkanSampler::new(&self.device)
            .map_err(|err| SamplerError::SamplerCreationError(err.into()))?;
        let sampler_id = sampler.id();
        self.samplers.insert(sampler_id, sampler);
        Ok(Sampler::new(sampler_id))
    }

    fn delete_image_sampler(
        &mut self,
        sampler: Sampler,
    ) -> Result<(), SamplerError> {
        let sampler = self
            .samplers
            .remove(&sampler.id())
            .ok_or(SamplerError::SamplerNotFound)?;
        sampler
            .delete_sampler(&self.device)
            .map_err(|err| SamplerError::SamplerDeletionError(err.into()))
    }

    fn create_framebuffer(
        &mut self,
        size: Vec2u,
        attachments: &[ImageView],
        renderpass: Renderpass,
    ) -> Result<Framebuffer, FramebufferError> {
        let att = attachments
            .iter()
            .map(|attachment| {
                self.image_views
                    .get(&attachment.id())
                    .ok_or(FramebufferError::ImageViewNotFound)
                    .map(|image_view| {
                        let o_image = self
                            .owned_images
                            .get(&image_view.image_id())
                            .map(|img| img as &dyn VulkanImage);
                        let s_image = self
                            .swapchain_images
                            .get(&image_view.image_id())
                            .map(|img| img as &dyn VulkanImage);
                        o_image
                            .or(s_image)
                            .map(|img| (image_view, img))
                            .ok_or(FramebufferError::ImageNotFound)
                    })
            })
            .collect::<Result<Result<Vec<_>, _>, _>>()??;
        let ren = self
            .renderpasses
            .get(&renderpass.id())
            .ok_or(FramebufferError::RenderpassNotFound)?;
        let framebuffer = VulkanFramebuffer::new(&self.device, &att, ren, size)
            .map_err(|err| {
                FramebufferError::FramebufferCreationError(err.into())
            })?;
        let framebuffer_id = framebuffer.id();
        self.framebuffers.insert(framebuffer_id, framebuffer);
        Ok(Framebuffer::new(framebuffer_id))
    }

    fn delete_framebuffer(
        &mut self,
        framebuffer: Framebuffer,
    ) -> Result<(), FramebufferError> {
        let framebuffer = self
            .framebuffers
            .remove(&framebuffer.id())
            .ok_or(FramebufferError::FramebufferNotFound)?;
        framebuffer.delete_framebuffer(&self.device).map_err(|err| {
            FramebufferError::FramebufferDeletionError(err.into())
        })
    }

    fn create_fence(&mut self, signaled: bool) -> Result<Fence, FenceError> {
        let fence = VulkanFence::new(&self.device, signaled)
            .map_err(|err| FenceError::FenceCreationError(err.into()))?;
        let fence_id = fence.id();
        self.fences.insert(fence_id, fence);
        Ok(Fence::new(fence_id))
    }

    fn wait_fence(&mut self, fence: Fence) -> Result<(), FenceError> {
        let fence = self
            .fences
            .get(&fence.id())
            .ok_or(FenceError::FenceNotFound)?;
        fence
            .wait(&self.device)
            .map_err(|err| FenceError::FenceWaitError(err.into()))
    }

    fn reset_fence(&mut self, fence: Fence) -> Result<(), FenceError> {
        let fence = self
            .fences
            .get(&fence.id())
            .ok_or(FenceError::FenceNotFound)?;
        fence
            .reset(&self.device)
            .map_err(|err| FenceError::FenceWaitError(err.into()))
    }

    fn delete_fence(&mut self, fence: Fence) -> Result<(), FenceError> {
        let fence = self
            .fences
            .remove(&fence.id())
            .ok_or(FenceError::FenceNotFound)?;
        fence
            .delete_fence(&self.device)
            .map_err(|err| FenceError::FenceWaitError(err.into()))
    }

    fn create_semaphore(&mut self) -> Result<Semaphore, SemaphoreError> {
        let semaphore = VulkanSemaphore::new(&self.device).map_err(|err| {
            SemaphoreError::SemaphoreCreationError(err.into())
        })?;
        let semaphore_id = semaphore.id();
        self.semaphores.insert(semaphore_id, semaphore);
        Ok(Semaphore::new(semaphore_id))
    }

    fn delete_semaphore(
        &mut self,
        semaphore: Semaphore,
    ) -> Result<(), SemaphoreError> {
        let semaphore = self
            .semaphores
            .remove(&semaphore.id())
            .ok_or(SemaphoreError::SemaphoreNotFound)?;
        semaphore
            .delete_semaphore(&self.device)
            .map_err(|err| SemaphoreError::SemaphoreDeletionError(err.into()))
    }
}
