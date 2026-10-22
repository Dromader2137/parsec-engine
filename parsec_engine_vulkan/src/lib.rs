//! Module responsible for interaction with the Vulkan API. (Incomplete, undocumented and subject
//! to change).

#![feature(linked_list_cursors)]

use std::collections::HashMap;

use parsec_engine::graphics::{
    backend::{BackendError, GraphicsBackend},
    buffer::{Buffer, BufferContent, BufferHandle, BufferUsage},
    command_list::{Command, CommandList},
    framebuffer::{Framebuffer, FramebufferHandle},
    gpu_cpu_fence::GpuToCpuFence,
    gpu_gpu_fence::GpuToGpuFence,
    image::{Image, ImageAspect, ImageFormat, ImageHandle, ImageUsage, ImageView, ImageViewHandle},
    pipeline::{
        Pipeline, PipelineHandle, PipelineOptions, PipelineResource, PipelineResourceBindingLayout,
        PipelineResourceHandle, PipelineResourceLayout, PipelineResourceLayoutHandle,
    },
    renderpass::{Renderpass, RenderpassAttachment, RenderpassHandle},
    sampler::{Sampler, SamplerHandle},
    shader_module::{ShaderHandle, ShaderModule, ShaderType},
    window::Window,
};
use parsec_engine_math::{ivec::Vec2i, uvec::Vec2u};

use crate::{
    allocator::{VulkanAllocator, VulkanMemoryProperties},
    buffer::VulkanBuffer,
    buffer_usage::VulkanBufferUsage,
    command_buffer::{VulkanCommandBuffer, VulkanCommandBufferBuilder, VulkanCommandPool},
    descriptor_set::{
        VulkanDescriptorPool, VulkanDescriptorPoolSize, VulkanDescriptorSet,
        VulkanDescriptorSetBinding, VulkanDescriptorSetLayout, VulkanDescriptorType,
    },
    device::VulkanDevice,
    fence::VulkanFence,
    framebuffer::VulkanFramebuffer,
    graphics_pipeline::{VulkanGraphicsPipeline, VulkanPipelineOptions, VulkanShaderStage},
    image::{
        VulkanImage, VulkanImageAspect, VulkanImageFormat, VulkanImageSize, VulkanImageUsage,
        VulkanImageView, VulkanOwnedImage,
    },
    instance::VulkanInstance,
    physical_device::VulkanPhysicalDevice,
    pipeline_stage::VulkanPipelineStage,
    queue::VulkanQueue,
    renderpass::{VulkanClearValue, VulkanRenderpass, VulkanRenderpassAttachment},
    sampler::VulkanSampler,
    semaphore::VulkanSemaphore,
    shader::VulkanShaderModule,
    surface::{VulkanInitialSurface, VulkanSurface},
    swapchain::VulkanSwapchain,
};

#[derive(Debug, thiserror::Error)]
enum VulkanBackendError {
    #[error("{0} not found")]
    NotFound(&'static str),
    #[error("invalid image size")]
    InvalidImageSize,
}

impl From<VulkanBackendError> for BackendError {
    fn from(err: VulkanBackendError) -> Self { BackendError::new(err) }
}

mod access;
mod allocation;
mod allocator;
mod barriers;
mod buffer;
mod buffer_usage;
mod command_buffer;
mod descriptor_set;
mod device;
mod fence;
mod format_size;
mod framebuffer;
mod graphics_pipeline;
mod image;
mod instance;
mod memory;
mod physical_device;
mod pipeline_stage;
mod queue;
mod renderpass;
mod sampler;
mod semaphore;
pub mod shader;
mod surface;
mod swapchain;
mod utils;

#[macro_export]
macro_rules! create_counter {
    ($name:ident) => {
        static $name: ::std::sync::LazyLock<parsec_engine::utils::id_counter::IdCounter> =
            ::std::sync::LazyLock::new(|| parsec_engine::utils::id_counter::IdCounter::default());
    };
}

#[allow(unused)]
pub struct VulkanBackend {
    instance: VulkanInstance,
    physical_device: VulkanPhysicalDevice,
    surface: VulkanSurface,
    device: VulkanDevice,
    command_pool: VulkanCommandPool,
    present_queue: VulkanQueue,
    descriptor_pool: VulkanDescriptorPool,
    allocator: VulkanAllocator,
    swapchain: VulkanSwapchain,
    images: HashMap<u32, Box<dyn VulkanImage>>,
    image_views: HashMap<u32, VulkanImageView>,
    samplers: HashMap<u32, VulkanSampler>,
    framebuffers: HashMap<u32, VulkanFramebuffer>,
    fences: HashMap<u32, VulkanFence>,
    semaphores: HashMap<u32, VulkanSemaphore>,
    command_buffers: HashMap<u32, VulkanCommandBuffer>,
    buffers: HashMap<u32, VulkanBuffer>,
    staging_buffers: HashMap<u32, u32>,
    shaders: HashMap<u32, VulkanShaderModule>,
    pipelines: HashMap<u32, VulkanGraphicsPipeline>,
    renderpasses: HashMap<u32, VulkanRenderpass>,
    descriptor_sets: HashMap<u32, VulkanDescriptorSet>,
    descriptor_set_layouts: HashMap<u32, VulkanDescriptorSetLayout>,
}

impl Drop for VulkanBackend {
    fn drop(&mut self) {
        self.device.wait_idle().unwrap();

        for (_, framebuffer) in self.framebuffers.drain() {
            framebuffer.destroy(&self.device);
        }
        for (_, renderpass) in self.renderpasses.drain() {
            renderpass.destroy(&self.device);
        }
        for (_, pipeline) in self.pipelines.drain() {
            pipeline.destroy(&self.device);
        }
        for (_, shader) in self.shaders.drain() {
            shader.destroy(&self.device);
        }
        for (_, layout) in self.descriptor_set_layouts.drain() {
            layout.destroy(&self.device);
        }
        self.descriptor_sets.clear();
        self.descriptor_pool.destroy(&self.device);
        for (_, sampler) in self.samplers.drain() {
            sampler.destroy(&self.device);
        }
        for (_, image_view) in self.image_views.drain() {
            image_view.destroy(&self.device);
        }
        for (_, image) in self.images.drain() {
            image.destroy(&self.device, &mut self.allocator);
        }
        for (_, buffer) in self.buffers.drain() {
            buffer.destroy(&self.device, &mut self.allocator);
        }
        for (_, fence) in self.fences.drain() {
            fence.destroy(&self.device);
        }
        for (_, semaphore) in self.semaphores.drain() {
            semaphore.destroy(&self.device);
        }

        self.swapchain.destroy();
        self.allocator.free_all(&self.device);

        self.command_buffers.clear();
        self.command_pool.destroy(&self.device);
        self.present_queue.destroy(&self.device);

        self.surface.destroy();
        self.device.destroy();
    }
}

impl GraphicsBackend for VulkanBackend {
    fn init(window: &Window) -> Result<Self, BackendError> {
        let instance = VulkanInstance::new(window).map_err(BackendError::new)?;
        let initial_surface =
            VulkanInitialSurface::new(&instance, window).map_err(BackendError::new)?;
        let physical_device =
            VulkanPhysicalDevice::new(&instance, &initial_surface).map_err(BackendError::new)?;
        let surface = VulkanSurface::from_initial_surface(initial_surface, &physical_device)
            .map_err(BackendError::new)?;
        let device = VulkanDevice::new(&instance, &physical_device).map_err(BackendError::new)?;
        let command_pool =
            VulkanCommandPool::new(&physical_device, &device).map_err(BackendError::new)?;
        let present_queue = VulkanQueue::present(&device, physical_device.queue_family_index());
        let descriptor_pool = VulkanDescriptorPool::new(&device, 4096, &[
            VulkanDescriptorPoolSize::new(4096, VulkanDescriptorType::UniformBuffer),
            VulkanDescriptorPoolSize::new(1024, VulkanDescriptorType::StorageBuffer),
            VulkanDescriptorPoolSize::new(16, VulkanDescriptorType::CombinedImageSampler),
        ])
        .map_err(BackendError::new)?;
        let allocator = VulkanAllocator::new();
        let (swapchain, swapchain_images) =
            VulkanSwapchain::new(&instance, window, &surface, &device, None)
                .map_err(BackendError::new)?;
        let mut images = HashMap::new();
        for swapchain_image in swapchain_images.iter() {
            let swapchain_image_id = swapchain_image.id();
            images.insert(
                swapchain_image_id,
                Box::new(swapchain_image.clone()) as Box<dyn VulkanImage>,
            );
        }

        Ok(VulkanBackend {
            instance,
            physical_device,
            surface,
            device,
            command_pool,
            present_queue,
            descriptor_pool,
            allocator,
            swapchain,
            images,
            image_views: HashMap::new(),
            samplers: HashMap::new(),
            framebuffers: HashMap::new(),
            fences: HashMap::new(),
            semaphores: HashMap::new(),
            command_buffers: HashMap::new(),
            buffers: HashMap::new(),
            staging_buffers: HashMap::new(),
            shaders: HashMap::new(),
            pipelines: HashMap::new(),
            renderpasses: HashMap::new(),
            descriptor_sets: HashMap::new(),
            descriptor_set_layouts: HashMap::new(),
        })
    }

    fn wait_idle(&self) { self.device.wait_idle().unwrap(); }

    fn get_surface_format(&self) -> ImageFormat { self.surface.format().general_image_format() }

    fn create_buffer(
        &mut self,
        data: BufferContent<'_>,
        buffer_usage: &[BufferUsage],
    ) -> Result<BufferHandle, BackendError> {
        let usage: Vec<_> = buffer_usage
            .iter()
            .map(|x| VulkanBufferUsage::new(*x))
            .collect();

        let size = data.data.len();

        let buffer = if size <= 256 {
            VulkanBuffer::from_vec(
                &self.device,
                &mut self.allocator,
                data,
                &usage,
                VulkanMemoryProperties::Host,
            )
            .map_err(BackendError::new)?
        } else {
            let staging = VulkanBuffer::from_vec(
                &self.device,
                &mut self.allocator,
                data,
                &[VulkanBufferUsage::TransferSrc],
                VulkanMemoryProperties::Host,
            )
            .map_err(BackendError::new)?;

            let mut usage = usage.to_vec();
            usage.push(VulkanBufferUsage::TransferDst);
            let device_local = VulkanBuffer::new(
                &self.device,
                &mut self.allocator,
                size as u64,
                usage.as_slice(),
                VulkanMemoryProperties::Device,
            )
            .map_err(BackendError::new)?;

            let mut cmd = VulkanCommandBuffer::new(&self.device, &self.command_pool)
                .map_err(BackendError::new)?;
            let mut builder = VulkanCommandBufferBuilder::new(&self.images, &self.buffers)
                .map_err(BackendError::new)?;
            builder.begin().map_err(BackendError::new)?;
            builder
                .copy_buffer_to_buffer(&staging, &device_local)
                .map_err(BackendError::new)?;
            builder.end().map_err(BackendError::new)?;
            builder
                .build(&self.device, &mut cmd)
                .map_err(BackendError::new)?;
            self.present_queue
                .submit(
                    &self.device,
                    &[],
                    &[],
                    &[&cmd],
                    &VulkanFence::null(),
                    VulkanPipelineStage::Transfer,
                    &mut self.images,
                )
                .map_err(BackendError::new)?;

            self.staging_buffers.insert(device_local.id(), staging.id());
            self.buffers.insert(staging.id(), staging);

            device_local
        };
        let buffer_id = buffer.id();
        self.buffers.insert(buffer_id, buffer);
        Ok(BufferHandle::new(buffer_id))
    }

    fn update_buffer(
        &mut self,
        buffer: BufferHandle,
        data: BufferContent<'_>,
    ) -> Result<(), BackendError> {
        let buffer_id = buffer.id();
        let buffer = self
            .buffers
            .get(&buffer_id)
            .ok_or(VulkanBackendError::NotFound("buffer"))?;
        if let Some(staging_id) = self.staging_buffers.get(&buffer_id) {
            let staging = self
                .buffers
                .get(staging_id)
                .ok_or(VulkanBackendError::NotFound("buffer"))?;

            staging
                .update(&self.device, data)
                .map_err(BackendError::new)?;

            let mut cmd = VulkanCommandBuffer::new(&self.device, &self.command_pool)
                .map_err(BackendError::new)?;
            let mut builder = VulkanCommandBufferBuilder::new(&self.images, &self.buffers)
                .map_err(BackendError::new)?;
            builder.begin().map_err(BackendError::new)?;
            builder
                .copy_buffer_to_buffer(staging, buffer)
                .map_err(BackendError::new)?;
            builder.end().map_err(BackendError::new)?;
            builder
                .build(&self.device, &mut cmd)
                .map_err(BackendError::new)?;
            self.present_queue
                .submit(
                    &self.device,
                    &[],
                    &[],
                    &[&cmd],
                    &VulkanFence::null(),
                    VulkanPipelineStage::Transfer,
                    &mut self.images,
                )
                .map_err(BackendError::new)?;
        } else {
            buffer
                .update(&self.device, data)
                .map_err(BackendError::new)?;
        }
        Ok(())
    }

    fn delete_buffer(&mut self, buffer: Buffer) -> Result<(), BackendError> {
        if let Some(staging_id) = self.staging_buffers.get(&buffer.id()) {
            let staging = self
                .buffers
                .remove(staging_id)
                .ok_or(VulkanBackendError::NotFound("buffer"))?;
            staging.destroy(&self.device, &mut self.allocator);
        }
        let buffer = self
            .buffers
            .remove(&buffer.id())
            .ok_or(VulkanBackendError::NotFound("buffer"))?;
        buffer.destroy(&self.device, &mut self.allocator);
        Ok(())
    }

    fn create_shader(
        &mut self,
        code: &[u32],
        shader_type: ShaderType,
    ) -> Result<ShaderHandle, BackendError> {
        let shader =
            VulkanShaderModule::new(&self.device, code, shader_type).map_err(BackendError::new)?;
        let shader_id = shader.id();
        self.shaders.insert(shader_id, shader);
        Ok(ShaderHandle::new(shader_id))
    }

    fn delete_shader(&mut self, shader: ShaderModule) -> Result<(), BackendError> {
        let shader = self
            .shaders
            .remove(&shader.handle().id())
            .ok_or(VulkanBackendError::NotFound("shader"))?;
        shader.destroy(&self.device);
        Ok(())
    }

    fn create_renderpass(
        &mut self,
        attachments: &[RenderpassAttachment],
    ) -> Result<RenderpassHandle, BackendError> {
        let renderpass = VulkanRenderpass::new(
            &self.device,
            &attachments
                .iter()
                .map(|x| {
                    (
                        VulkanRenderpassAttachment::new(*x),
                        VulkanClearValue::new(x.clear_value),
                    )
                })
                .collect::<Vec<_>>(),
        )
        .map_err(BackendError::new)?;
        let rendrepass_id = renderpass.id();
        self.renderpasses.insert(rendrepass_id, renderpass);
        Ok(RenderpassHandle::new(rendrepass_id))
    }

    fn delete_renderpass(&mut self, renderpass: Renderpass) -> Result<(), BackendError> {
        let renderpass = self
            .renderpasses
            .remove(&renderpass.id())
            .ok_or(VulkanBackendError::NotFound("renderpass"))?;
        renderpass.destroy(&self.device);
        Ok(())
    }

    fn create_pipeline(
        &mut self,
        vertex_shader: ShaderHandle,
        fragment_shader: ShaderHandle,
        renderpass: RenderpassHandle,
        binding_layouts: &[PipelineResourceLayoutHandle],
        options: PipelineOptions,
    ) -> Result<PipelineHandle, BackendError> {
        let vsm = self
            .shaders
            .get(&vertex_shader.id())
            .ok_or(VulkanBackendError::NotFound("shader"))?;
        let fsm = self
            .shaders
            .get(&fragment_shader.id())
            .ok_or(VulkanBackendError::NotFound("shader"))?;
        let ren = self
            .renderpasses
            .get(&renderpass.id())
            .ok_or(VulkanBackendError::NotFound("renderpass"))?;
        let dsl = binding_layouts
            .iter()
            .map(|x| {
                self.descriptor_set_layouts
                    .get(&x.id())
                    .ok_or(VulkanBackendError::NotFound("pipeline resource layout"))
                    .cloned()
            })
            .collect::<Result<Vec<_>, _>>()?;
        let pipeline = VulkanGraphicsPipeline::new(
            &self.device,
            ren,
            vsm,
            fsm,
            &dsl,
            VulkanPipelineOptions::new(options),
        )
        .map_err(BackendError::new)?;
        let pipeline_id = pipeline.id();
        self.pipelines.insert(pipeline_id, pipeline);
        Ok(PipelineHandle::new(pipeline_id))
    }

    fn delete_pipeline(&mut self, pipeline: Pipeline) -> Result<(), BackendError> {
        let pip = self
            .pipelines
            .remove(&pipeline.id())
            .ok_or(VulkanBackendError::NotFound("pipeline"))?;
        pip.destroy(&self.device);
        Ok(())
    }

    fn create_pipeline_resource_layout(
        &mut self,
        subbindings: &[PipelineResourceBindingLayout],
    ) -> Result<PipelineResourceLayoutHandle, BackendError> {
        let set_bindings = subbindings
            .iter()
            .enumerate()
            .map(|(id, binding)| {
                VulkanDescriptorSetBinding::new(
                    id as u32,
                    VulkanDescriptorType::new(binding.binding_type),
                    &binding
                        .shader_stages
                        .iter()
                        .map(|x| VulkanShaderStage::new(*x))
                        .collect::<Vec<_>>(),
                )
            })
            .collect::<Vec<_>>();
        let dsl = VulkanDescriptorSetLayout::new(&self.device, set_bindings)
            .map_err(BackendError::new)?;
        let dsl_id = dsl.id();
        self.descriptor_set_layouts.insert(dsl_id, dsl);
        Ok(PipelineResourceLayoutHandle::new(dsl_id))
    }

    fn delete_pipeline_resource_layout(
        &mut self,
        layout: PipelineResourceLayout,
    ) -> Result<(), BackendError> {
        let dsl = self
            .descriptor_set_layouts
            .remove(&layout.id())
            .ok_or(VulkanBackendError::NotFound("pipeline resource layout"))?;
        dsl.destroy(&self.device);
        Ok(())
    }

    fn create_pipeline_resource(
        &mut self,
        pipeline_layout: PipelineResourceLayoutHandle,
    ) -> Result<PipelineResourceHandle, BackendError> {
        let dsl = self
            .descriptor_set_layouts
            .get(&pipeline_layout.id())
            .ok_or(VulkanBackendError::NotFound("pipeline resource layout"))?;
        let ds = VulkanDescriptorSet::new(&self.device, dsl, &self.descriptor_pool)
            .map_err(BackendError::new)?;
        let ds_id = ds.id();
        self.descriptor_sets.insert(ds_id, ds);
        Ok(PipelineResourceHandle::new(ds_id))
    }

    fn delete_pipeline_resource(&mut self, resrouce: PipelineResource) -> Result<(), BackendError> {
        let ds = self
            .descriptor_sets
            .remove(&resrouce.id())
            .ok_or(VulkanBackendError::NotFound("pipeline resource"))?;
        ds.free(&self.device, &self.descriptor_pool)
            .map_err(BackendError::new)?;
        Ok(())
    }

    fn bind_buffer(
        &mut self,
        pipeline_binding: PipelineResourceHandle,
        buffer: BufferHandle,
        index: u32,
    ) -> Result<(), BackendError> {
        let ds = self
            .descriptor_sets
            .get_mut(&pipeline_binding.id())
            .ok_or(VulkanBackendError::NotFound("pipeline resource"))?;
        let dsl = self
            .descriptor_set_layouts
            .get(&ds.descriptor_layout_id())
            .ok_or(VulkanBackendError::NotFound("pipeline resource layout"))?;
        let buf = self
            .buffers
            .get(&buffer.id())
            .ok_or(VulkanBackendError::NotFound("buffer"))?;
        ds.bind_buffer(buf, &self.device, dsl, index)
            .map_err(BackendError::new)
    }

    fn bind_sampler(
        &mut self,
        pipeline_binding: PipelineResourceHandle,
        sampler: SamplerHandle,
        image_view: ImageViewHandle,
        index: u32,
    ) -> Result<(), BackendError> {
        let ds = self
            .descriptor_sets
            .get_mut(&pipeline_binding.id())
            .ok_or(VulkanBackendError::NotFound("pipeline resource"))?;
        let dsl = self
            .descriptor_set_layouts
            .get(&ds.descriptor_layout_id())
            .ok_or(VulkanBackendError::NotFound("pipeline resource layout"))?;
        let sam = self
            .samplers
            .get(&sampler.id())
            .ok_or(VulkanBackendError::NotFound("sampler"))?;
        let imv = self
            .image_views
            .get(&image_view.id())
            .ok_or(VulkanBackendError::NotFound("image view"))?;
        ds.bind_image_view(imv, sam, &self.device, dsl, index)
            .map_err(BackendError::new)
    }

    fn create_command_list(&mut self) -> Result<CommandList, BackendError> {
        let comman_buffer = VulkanCommandBuffer::new(&self.device, &self.command_pool)
            .map_err(BackendError::new)?;
        let command_buffer_id = comman_buffer.id();
        self.command_buffers
            .insert(command_buffer_id, comman_buffer);
        Ok(CommandList::new(command_buffer_id))
    }

    fn submit_commands(
        &mut self,
        command_list: &CommandList,
        wait_semaphores: &[GpuToGpuFence],
        signal_semaphores: &[GpuToGpuFence],
        signal_fence: GpuToCpuFence,
    ) -> Result<(), BackendError> {
        let command_buffer = self
            .command_buffers
            .get_mut(&command_list.id())
            .ok_or(VulkanBackendError::NotFound("command list"))?;
        let mut builder = VulkanCommandBufferBuilder::new(&self.images, &self.buffers)
            .map_err(BackendError::new)?;
        let ws = wait_semaphores
            .iter()
            .map(|x| {
                self.semaphores
                    .get(&x.id())
                    .ok_or(VulkanBackendError::NotFound("semaphore"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let ss = signal_semaphores
            .iter()
            .map(|x| {
                self.semaphores
                    .get(&x.id())
                    .ok_or(VulkanBackendError::NotFound("semaphore"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        let fen = self
            .fences
            .get(&signal_fence.id())
            .ok_or(VulkanBackendError::NotFound("fence"))?;

        for command in command_list.commands().iter() {
            match command {
                Command::Begin => builder.begin().map_err(BackendError::new)?,
                Command::End => builder.end().map_err(BackendError::new)?,
                Command::BeginRenderpass(renderpass, framebuffer) => {
                    let r = self.renderpasses.get(&renderpass.id()).unwrap();
                    let f = self.framebuffers.get(&framebuffer.id()).unwrap();
                    builder.begin_renderpass(r, f).map_err(BackendError::new)?;
                    builder
                        .set_viewports(f.dimensions())
                        .map_err(BackendError::new)?;
                    builder
                        .set_scissor(f.dimensions(), Vec2i::ZERO)
                        .map_err(BackendError::new)?
                },
                Command::SetViewport(dimension) => builder
                    .set_viewports(*dimension)
                    .map_err(BackendError::new)?,
                Command::SetScissor(dimension, offest) => builder
                    .set_scissor(*dimension, *offest)
                    .map_err(BackendError::new)?,
                Command::EndRenderpass => builder.end_renderpass().map_err(BackendError::new)?,
                Command::BindGraphicsPipeline(pipeline) => {
                    let p = self.pipelines.get(&pipeline.id()).unwrap();
                    builder
                        .bind_graphics_pipeline(p)
                        .map_err(BackendError::new)?
                },
                Command::BindPipelineBinding(pipeline_binding, idx) => {
                    let ds = self.descriptor_sets.get(&pipeline_binding.id()).unwrap();
                    builder
                        .bind_descriptor_set(ds, *idx)
                        .map_err(BackendError::new)?
                },
                Command::BindVertexBuffer(buffer) => {
                    let b = self.buffers.get(&buffer.id()).unwrap();

                    builder.bind_vertex_buffer(b).map_err(BackendError::new)?
                },
                Command::BindIndexBuffer(buffer) => {
                    let b = self.buffers.get(&buffer.id()).unwrap();

                    builder.bind_index_buffer(b).map_err(BackendError::new)?
                },
                Command::Draw(vc, ic, fv, fi) => {
                    builder
                        .draw(*vc, *ic, *fv, *fi)
                        .map_err(BackendError::new)?;
                },
                Command::DrawIndexed(idc, inc, fid, vo, fin) => {
                    builder
                        .draw_indexed(*idc, *inc, *fid, *vo, *fin)
                        .map_err(BackendError::new)?;
                },
                Command::CopyBufferToImage(buffer, image, image_size, image_offset) => {
                    let b = self.buffers.get(&buffer.id()).unwrap();
                    let i = self.images.get(&image.id()).unwrap();
                    builder
                        .copy_buffer_to_image(b, i, *image_size, *image_offset)
                        .map_err(BackendError::new)?
                },
                Command::CopyBufferToBuffer(src, dst) => {
                    let s = self.buffers.get(&src.id()).unwrap();
                    let d = self.buffers.get(&dst.id()).unwrap();
                    builder
                        .copy_buffer_to_buffer(s, d)
                        .map_err(BackendError::new)?
                },
            };
        }

        builder
            .build(&self.device, command_buffer)
            .map_err(BackendError::new)?;

        self.present_queue
            .submit(
                &self.device,
                &ws,
                &ss,
                &[command_buffer],
                fen,
                VulkanPipelineStage::ColorAttachmentOutput,
                &mut self.images,
            )
            .map_err(BackendError::new)
    }

    fn handle_resize(&mut self, window: &Window) -> Result<(), BackendError> {
        let (swapchain, swapchain_images) = VulkanSwapchain::new(
            &self.instance,
            window,
            &self.surface,
            &self.device,
            Some(&self.swapchain),
        )
        .map_err(BackendError::new)?;
        self.swapchain.destroy();
        self.swapchain = swapchain;
        for swapchain_image in swapchain_images.iter() {
            let swapchain_image_id = swapchain_image.id();
            self.images
                .insert(swapchain_image_id, Box::new(swapchain_image.clone()));
        }
        Ok(())
    }

    fn present_images(&mut self) -> Vec<ImageHandle> {
        self.swapchain
            .swapchain_image_ids()
            .iter()
            .map(|x| ImageHandle::new(*x))
            .collect()
    }

    fn start_frame(&mut self, signal_semaphore: GpuToGpuFence) -> Result<u32, BackendError> {
        let ss = self
            .semaphores
            .get(&signal_semaphore.id())
            .ok_or(VulkanBackendError::NotFound("semaphore"))?;

        self.swapchain
            .acquire_next_image(ss, &VulkanFence::null())
            .map_err(BackendError::new)
            .map(|val| val.0)
    }

    fn end_frame(
        &mut self,
        wait_semaphores: &[GpuToGpuFence],
        present_image_index: u32,
    ) -> Result<(), BackendError> {
        let ws = wait_semaphores
            .iter()
            .map(|x| {
                self.semaphores
                    .get(&x.id())
                    .ok_or(VulkanBackendError::NotFound("semaphore"))
            })
            .collect::<Result<Vec<_>, _>>()?;
        self.swapchain
            .present(&self.present_queue, &ws, present_image_index)
            .map_err(BackendError::new)
    }

    fn create_image(
        &mut self,
        size: Vec2u,
        format: ImageFormat,
        aspect: ImageAspect,
        usage: &[ImageUsage],
    ) -> Result<ImageHandle, BackendError> {
        let usage: Vec<_> = usage.iter().map(|x| VulkanImageUsage::new(*x)).collect();
        let aspect = VulkanImageAspect::new(aspect);
        let size = VulkanImageSize::new(size).ok_or(VulkanBackendError::InvalidImageSize)?;
        let image = VulkanOwnedImage::new(
            &self.device,
            &mut self.allocator,
            size,
            VulkanImageFormat::new(format),
            &usage,
            aspect,
            VulkanMemoryProperties::Device,
        )
        .map_err(BackendError::new)?;
        let image_id = image.id();
        self.images.insert(image_id, Box::new(image));
        Ok(ImageHandle::new(image_id))
    }

    fn load_image_from_buffer(
        &mut self,
        buffer: BufferHandle,
        image: ImageHandle,
        image_size: Vec2u,
        image_offset: Vec2u,
    ) -> Result<(), BackendError> {
        let buf = self
            .buffers
            .get(&buffer.id())
            .ok_or(VulkanBackendError::NotFound("buffer"))?;
        let mut cmd = VulkanCommandBuffer::new(&self.device, &self.command_pool)
            .map_err(BackendError::new)?;
        let mut builder = VulkanCommandBufferBuilder::new(&self.images, &self.buffers)
            .map_err(BackendError::new)?;
        let img = self
            .images
            .get(&image.id())
            .ok_or(VulkanBackendError::NotFound("image"))?;
        builder.begin().map_err(BackendError::new)?;
        builder
            .copy_buffer_to_image(buf, img, image_size, image_offset)
            .map_err(BackendError::new)?;
        builder.end().map_err(BackendError::new)?;
        builder
            .build(&self.device, &mut cmd)
            .map_err(BackendError::new)?;
        let fence = VulkanFence::new(&self.device, false).map_err(BackendError::new)?;
        self.present_queue
            .submit(
                &self.device,
                &[],
                &[],
                &[&cmd],
                &fence,
                VulkanPipelineStage::Transfer,
                &mut self.images,
            )
            .map_err(BackendError::new)?;
        fence.wait(&self.device).map_err(BackendError::new)?;
        fence.destroy(&self.device);
        Ok(())
    }

    fn delete_image(&mut self, image: Image) -> Result<(), BackendError> {
        let result = self
            .images
            .remove(&image.id())
            .ok_or(VulkanBackendError::NotFound("image"))?;
        result.destroy(&self.device, &mut self.allocator);
        Ok(())
    }

    fn create_image_view(&mut self, image: ImageHandle) -> Result<ImageViewHandle, BackendError> {
        let im = self
            .images
            .get(&image.id())
            .ok_or(VulkanBackendError::NotFound("image"))?;
        let iv = VulkanImageView::from_image(&self.device, im).map_err(BackendError::new)?;
        let iv_id = iv.id();
        self.image_views.insert(iv_id, iv);
        Ok(ImageViewHandle::new(iv_id))
    }

    fn delete_image_view(&mut self, image_view: ImageView) -> Result<(), BackendError> {
        let image_view = self
            .image_views
            .remove(&image_view.id())
            .ok_or(VulkanBackendError::NotFound("image view"))?;
        image_view.destroy(&self.device);
        Ok(())
    }

    fn create_image_sampler(&mut self) -> Result<SamplerHandle, BackendError> {
        let sampler = VulkanSampler::new(&self.device).map_err(BackendError::new)?;
        let sampler_id = sampler.id();
        self.samplers.insert(sampler_id, sampler);
        Ok(SamplerHandle::new(sampler_id))
    }

    fn delete_image_sampler(&mut self, sampler: Sampler) -> Result<(), BackendError> {
        let sampler = self
            .samplers
            .remove(&sampler.id())
            .ok_or(VulkanBackendError::NotFound("sampler"))?;
        sampler.destroy(&self.device);
        Ok(())
    }

    fn create_framebuffer(
        &mut self,
        size: Vec2u,
        attachments: &[ImageViewHandle],
        renderpass: RenderpassHandle,
    ) -> Result<FramebufferHandle, BackendError> {
        let att = attachments
            .iter()
            .map(|attachment| {
                self.image_views
                    .get(&attachment.id())
                    .ok_or(VulkanBackendError::NotFound("image view"))
                    .map(|image_view| {
                        self.images
                            .get(&image_view.image_id())
                            .map(|img| (image_view, img))
                            .ok_or(VulkanBackendError::NotFound("image"))
                    })
            })
            .collect::<Result<Result<Vec<_>, _>, _>>()??;
        let ren = self
            .renderpasses
            .get(&renderpass.id())
            .ok_or(VulkanBackendError::NotFound("renderpass"))?;
        let framebuffer =
            VulkanFramebuffer::new(&self.device, &att, ren, size).map_err(BackendError::new)?;
        let framebuffer_id = framebuffer.id();
        self.framebuffers.insert(framebuffer_id, framebuffer);
        Ok(FramebufferHandle::new(framebuffer_id))
    }

    fn delete_framebuffer(&mut self, framebuffer: Framebuffer) -> Result<(), BackendError> {
        let framebuffer = self
            .framebuffers
            .remove(&framebuffer.id())
            .ok_or(VulkanBackendError::NotFound("framebuffer"))?;
        framebuffer.destroy(&self.device);
        Ok(())
    }

    fn create_gpu_to_cpu_fence(&mut self, signaled: bool) -> Result<GpuToCpuFence, BackendError> {
        let fence = VulkanFence::new(&self.device, signaled).map_err(BackendError::new)?;
        let fence_id = fence.id();
        self.fences.insert(fence_id, fence);
        Ok(GpuToCpuFence::new(fence_id))
    }

    fn wait_gpu_to_cpu_fence(&mut self, fence: GpuToCpuFence) -> Result<(), BackendError> {
        let fence = self
            .fences
            .get(&fence.id())
            .ok_or(VulkanBackendError::NotFound("fence"))?;
        fence.wait(&self.device).map_err(BackendError::new)
    }

    fn reset_gpu_to_cpu_fence(&mut self, fence: GpuToCpuFence) -> Result<(), BackendError> {
        let fence = self
            .fences
            .get(&fence.id())
            .ok_or(VulkanBackendError::NotFound("fence"))?;
        fence.reset(&self.device).map_err(BackendError::new)
    }

    fn delete_gpu_to_cpu_fence(&mut self, fence: GpuToCpuFence) -> Result<(), BackendError> {
        let fence = self
            .fences
            .remove(&fence.id())
            .ok_or(VulkanBackendError::NotFound("fence"))?;
        fence.destroy(&self.device);
        Ok(())
    }

    fn create_gpu_to_gpu_fence(&mut self) -> Result<GpuToGpuFence, BackendError> {
        let semaphore = VulkanSemaphore::new(&self.device).map_err(BackendError::new)?;
        let semaphore_id = semaphore.id();
        self.semaphores.insert(semaphore_id, semaphore);
        Ok(GpuToGpuFence::new(semaphore_id))
    }

    fn delete_gpu_to_gpu_fence(&mut self, semaphore: GpuToGpuFence) -> Result<(), BackendError> {
        let semaphore = self
            .semaphores
            .remove(&semaphore.id())
            .ok_or(VulkanBackendError::NotFound("semaphore"))?;
        semaphore.destroy(&self.device);
        Ok(())
    }
}
