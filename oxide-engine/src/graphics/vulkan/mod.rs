//! Module responsible for interaction with the Vulkan API. (Incomplete, undocumented and subject
//! to change).

use std::collections::HashMap;

use crate::graphics::{
    GraphicsError,
    backend::GraphicsBackend,
    buffer::{Buffer, BufferError, BufferUsage},
    command_list::{CommandList, CommandListError},
    fence::Fence,
    framebuffer::Framebuffer,
    image::{Image, ImageFormat, ImageUsage, ImageView},
    pipeline::{
        Pipeline, PipelineBinding, PipelineBindingLayout, PipelineError,
        PipelineSubbindingLayout,
    },
    renderer::{DefaultVertex, RendererError},
    renderpass::{Renderpass, RenderpassError},
    semaphore::Semaphore,
    shader::{Shader, ShaderError, ShaderType},
    swapchain::{Swapchain, SwapchainError},
    vulkan::{
        buffer::{VulkanBuffer, VulkanBufferError, VulkanBufferUsage},
        command_buffer::{
            VulkanCommandBuffer, VulkanCommandBufferError, VulkanCommandPool,
            VulkanCommandPoolError,
        },
        descriptor_set::{
            DescriptorError, DescriptorType, VulkanDescriptorPool,
            VulkanDescriptorPoolSize, VulkanDescriptorSet,
            VulkanDescriptorSetBinding, VulkanDescriptorSetLayout,
        },
        device::{VulkanDevice, VulkanDeviceError},
        fence::{VulkanFence, VulkanFenceError},
        framebuffer::{VulkanFramebuffer, VulkanFramebufferError},
        graphics_pipeline::{
            VulkanGraphicsPipeline, VulkanGraphicsPipelineError,
        },
        image::{
            VulkanImage, VulkanImageError, VulkanImageInfo, VulkanImageView,
            VulkanOwnedImage, VulkanSwapchainImage,
        },
        instance::{VulkanInstance, VulkanInstanceError},
        physical_device::{VulkanPhysicalDevice, VulkanPhysicalDeviceError},
        queue::{VulkanQueue, VulkanQueueError},
        renderpass::{VulkanRenderpass, VulkanRenderpassError},
        semaphore::{VulkanSemaphore, VulkanSemaphoreError},
        shader::{VulkanShaderError, VulkanShaderModule},
        surface::{VulkanInitialSurface, VulkanSurface, VulkanSurfaceError},
        swapchain::{VulkanSwapchain, VulkanSwapchainError},
    },
    window::Window,
};

pub mod allocation;
pub mod allocator;
pub mod buffer;
pub mod command_buffer;
pub mod context;
pub mod descriptor_set;
pub mod device;
pub mod fence;
pub mod format_size;
pub mod framebuffer;
pub mod graphics_pipeline;
pub mod image;
pub mod instance;
pub mod physical_device;
pub mod queue;
pub mod renderpass;
pub mod semaphore;
pub mod shader;
pub mod surface;
pub mod swapchain;

#[derive(Debug)]
pub enum VulkanError {
    VulkanInstanceError(VulkanInstanceError),
    VulkanPhysicalDeviceError(VulkanPhysicalDeviceError),
    VulkanSurfaceError(VulkanSurfaceError),
    VulkanDeviceError(VulkanDeviceError),
    VulkanQueueError(VulkanQueueError),
    VulkanSwapchainError(VulkanSwapchainError),
    VulkanImageError(VulkanImageError),
    VulkanFramebufferError(VulkanFramebufferError),
    VulkanRenderpassError(VulkanRenderpassError),
    VulkanCommandBufferError(VulkanCommandBufferError),
    VulkanCommandPoolError(VulkanCommandPoolError),
    VulkanFenceError(VulkanFenceError),
    VulkanSemaphoreError(VulkanSemaphoreError),
    VulkanShaderError(VulkanShaderError),
    VulkanGraphicsPipelineError(VulkanGraphicsPipelineError),
    VulkanBufferError(VulkanBufferError),
    DescriptorError(DescriptorError),
    RendererError(RendererError),
}

impl From<VulkanError> for GraphicsError {
    fn from(value: VulkanError) -> Self { GraphicsError::VulkanError(value) }
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
    swapchains: HashMap<u32, VulkanSwapchain>,
    swapchain_images: HashMap<u32, VulkanSwapchainImage>,
    owned_images: HashMap<u32, VulkanOwnedImage>,
    image_views: HashMap<u32, VulkanImageView>,
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
    fn init(window: &Window) -> VulkanBackend {
        let instance = VulkanInstance::new(&window).unwrap();
        let initial_surface =
            VulkanInitialSurface::new(&instance, &window).unwrap();
        let physical_device =
            VulkanPhysicalDevice::new(&instance, &initial_surface).unwrap();
        let surface = VulkanSurface::from_initial_surface(
            initial_surface,
            &physical_device,
        )
        .unwrap();
        let device =
            VulkanDevice::new(&instance, &physical_device, &surface).unwrap();
        let command_pool =
            VulkanCommandPool::new(&physical_device, &device).unwrap();
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
        .unwrap();
        VulkanBackend {
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
        }
    }

    fn wait_idle(&self) { self.device.wait_idle().unwrap(); }

    fn create_buffer<T: Clone + Copy>(
        &mut self,
        data: &[T],
        buffer_usage: &[BufferUsage],
    ) -> Result<Buffer, BufferError> {
        let mut usage = VulkanBufferUsage::empty();
        buffer_usage.iter().for_each(|x| usage |= (*x).into());
        let buffer = VulkanBuffer::from_vec(&self.device, data, usage).unwrap();
        let buffer_id = buffer.id();
        self.buffers.insert(buffer_id, buffer);
        Ok(Buffer::new(buffer_id))
    }

    fn update_buffer<T: Clone + Copy>(
        &mut self,
        buffer: Buffer,
        data: &[T],
    ) -> Result<(), BufferError> {
        let buffer_id = buffer.id();
        let buffer = self
            .buffers
            .get(&buffer_id)
            .ok_or(BufferError::BufferNotFound)?;
        buffer.update(&self.device, data).unwrap();
        Ok(())
    }

    fn create_shader(
        &mut self,
        code: &[u32],
        shader_type: ShaderType,
    ) -> Result<Shader, ShaderError> {
        let shader =
            VulkanShaderModule::new(&self.device, code, shader_type).unwrap();
        let shader_id = shader.id();
        self.shaders.insert(shader_id, shader);
        Ok(Shader::new(shader_id))
    }

    fn create_renderpass(&mut self) -> Result<Renderpass, RenderpassError> {
        let renderpass =
            VulkanRenderpass::new(&self.surface, &self.device).unwrap();
        let rendrepass_id = renderpass.id();
        self.renderpasses.insert(rendrepass_id, renderpass);
        Ok(Renderpass::new(rendrepass_id))
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
        let dsl =
            VulkanDescriptorSetLayout::new(&self.device, set_bindings).unwrap();
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
        let fra = self
            .framebuffers
            .get(&0)
            .ok_or(PipelineError::FramebufferNotFound)?;
        let dsl = binding_layouts
            .iter()
            .map(|x| self.descriptor_set_layouts.get(&x.id()).unwrap().clone())
            .collect::<Vec<_>>();
        let pipeline = VulkanGraphicsPipeline::new::<DefaultVertex>(
            &self.device,
            ren,
            fra,
            vsm,
            fsm,
            &dsl,
        )
        .unwrap();
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
            .unwrap();
        let ds =
            VulkanDescriptorSet::new(&self.device, dsl, &self.descriptor_pool)
                .unwrap();
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
            .unwrap();
        let buf = self.buffers.get(&buffer.id()).unwrap();
        ds.bind_buffer(buf, &self.device, dsl, index).unwrap();
        Ok(())
    }

    fn create_command_list(&mut self) -> Result<CommandList, CommandListError> {
        let comman_buffer =
            VulkanCommandBuffer::new(&self.device, &self.command_pool).unwrap();
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
        command_buffer.begin(&self.device).unwrap();
        Ok(())
    }

    fn command_end(
        &mut self,
        command_list: CommandList,
    ) -> Result<(), CommandListError> {
        let command_buffer = self
            .command_buffers
            .get(&command_list.id())
            .ok_or(CommandListError::CommandListNotFound)?;
        command_buffer.end(&self.device).unwrap();
        Ok(())
    }

    fn command_reset(
        &mut self,
        command_list: CommandList,
    ) -> Result<(), CommandListError> {
        let command_buffer = self
            .command_buffers
            .get(&command_list.id())
            .ok_or(CommandListError::CommandListNotFound)?;
        command_buffer.reset(&self.device).unwrap();
        Ok(())
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
        let fra = self.framebuffers.get(&framebuffer.id()).unwrap();
        let ren = self.renderpasses.get(&renderpass.id()).unwrap();
        command_buffer
            .begin_renderpass(&self.device, fra, ren)
            .unwrap();
        command_buffer
            .set_viewports(&self.device, fra, ren)
            .unwrap();
        command_buffer.set_scissor(&self.device, fra, ren).unwrap();
        Ok(())
    }

    fn command_end_renderpass(
        &mut self,
        command_list: CommandList,
    ) -> Result<(), CommandListError> {
        let command_buffer = self
            .command_buffers
            .get(&command_list.id())
            .ok_or(CommandListError::CommandListNotFound)?;
        command_buffer.end_renderpass(&self.device).unwrap();
        Ok(())
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
        let pip = self.pipelines.get(&pipeline.id()).unwrap();
        command_buffer
            .bind_graphics_pipeline(&self.device, pip)
            .unwrap();
        Ok(())
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
        let ds = self.descriptor_sets.get(&binding.id()).unwrap();
        let pip = self.pipelines.get(&pipeline.id()).unwrap();
        command_buffer
            .bind_descriptor_set(&self.device, ds, pip, binding_index)
            .unwrap();
        Ok(())
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
        let vb = self.buffers.get(&vertex_buffer.id()).unwrap();
        let ib = self.buffers.get(&index_buffer.id()).unwrap();
        command_buffer.bind_vertex_buffer(&self.device, vb).unwrap();
        command_buffer.bind_index_buffer(&self.device, ib).unwrap();
        command_buffer
            .draw_indexed(&self.device, ib.len, 1, 0, 0, 1)
            .unwrap();
        Ok(())
    }

    fn submit_commands(
        &mut self,
        command_list: CommandList,
        wait_semaphores: &[Semaphore],
        signal_semaphores: &[Semaphore],
        signal_fence: Fence,
    ) {
        let command_buffer =
            self.command_buffers.get(&command_list.id()).unwrap();
        let ws = wait_semaphores
            .iter()
            .map(|x| self.semaphores.get(&x.id()).unwrap())
            .collect::<Vec<_>>();
        let ss = signal_semaphores
            .iter()
            .map(|x| self.semaphores.get(&x.id()).unwrap())
            .collect::<Vec<_>>();
        let fen = self.fences.get(&signal_fence.id()).unwrap();

        self.present_queue
            .submit(&self.device, &ws, &ss, &[command_buffer], &fen)
            .unwrap();
    }

    fn create_swapchain(
        &mut self,
        window: &Window,
        old_swapchain: Option<Swapchain>,
    ) -> Result<(Swapchain, Vec<Image>), SwapchainError> {
        let os = old_swapchain.map(|x| self.swapchains.get(&x.id()).unwrap());
        let (swapchain, swapchain_images) = VulkanSwapchain::new(
            &self.instance,
            &self.physical_device,
            window,
            &self.surface,
            &self.device,
            os,
        )
        .unwrap();
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

    fn delete_swapchain(&mut self, swapchain: Swapchain) {
        self.swapchains.remove(&swapchain.id()).unwrap();
    }

    fn next_image_id(
        &mut self,
        swapchain: Swapchain,
        signal_semaphore: Semaphore,
    ) -> Result<u32, SwapchainError> {
        let ss = self.semaphores.get(&signal_semaphore.id()).unwrap();

        self.swapchains
            .get(&swapchain.id())
            .unwrap()
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
            .map(|x| self.semaphores.get(&x.id()).unwrap())
            .collect::<Vec<_>>();
        self.swapchains
            .get(&swapchain.id())
            .unwrap()
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
        size: (u32, u32),
        format: ImageFormat,
        usage: ImageUsage,
    ) -> Image {
        let image = VulkanOwnedImage::new(&self.device, VulkanImageInfo {
            format: format.into(),
            size,
            usage: usage.into(),
        })
        .unwrap();
        let image_id = image.id();
        self.owned_images.insert(image_id, image);
        Image::new(image_id)
    }

    fn delete_image(&mut self, image: Image) {
        let result = self.owned_images.remove(&image.id());
        if let Some(image) = result {
            unsafe {
                self.device
                    .get_device_raw()
                    .destroy_image(*image.get_image_raw(), None);
            }
        } else {
            self.swapchain_images.remove(&image.id()).unwrap();
        }
    }

    fn create_image_view(&mut self, image: Image) -> ImageView {
        let oim = self
            .owned_images
            .get(&image.id())
            .map(|x| VulkanImageView::from_image(&self.device, x).unwrap());
        let sim = self
            .swapchain_images
            .get(&image.id())
            .map(|x| VulkanImageView::from_image(&self.device, x).unwrap());
        let im = oim.or(sim).unwrap();
        let im_id = im.id();
        self.image_views.insert(im_id, im);
        ImageView::new(im_id)
    }

    fn delete_image_view(&mut self, image_view: ImageView) {
        let image_view = self.image_views.remove(&image_view.id()).unwrap();
        unsafe {
            self.device
                .get_device_raw()
                .destroy_image_view(*image_view.get_image_view_raw(), None);
        }
    }

    fn create_framebuffer(
        &mut self,
        window: &Window,
        color_view: ImageView,
        depth_view: ImageView,
        renderpass: Renderpass,
    ) -> Framebuffer {
        let cv = self.image_views.get(&color_view.id()).unwrap();
        let dv = self.image_views.get(&depth_view.id()).unwrap();
        let ren = self.renderpasses.get(&renderpass.id()).unwrap();
        let framebuffer =
            VulkanFramebuffer::new(window, &self.device, cv, dv, ren).unwrap();
        let framebuffer_id = framebuffer.id();
        self.framebuffers.insert(framebuffer_id, framebuffer);
        Framebuffer::new(framebuffer_id)
    }

    fn delete_framebuffer(&mut self, framebuffer: Framebuffer) {
        let framebuffer = self.framebuffers.remove(&framebuffer.id()).unwrap();
        unsafe {
            self.device
                .get_device_raw()
                .destroy_framebuffer(*framebuffer.get_framebuffer_raw(), None);
        }
    }

    fn create_fence(&mut self, signaled: bool) -> Fence {
        let fence = VulkanFence::new(&self.device, signaled).unwrap();
        let fence_id = fence.id();
        self.fences.insert(fence_id, fence);
        Fence::new(fence_id)
    }

    fn wait_fence(&mut self, fence: Fence) {
        let fence = self.fences.get(&fence.id()).unwrap();
        fence.wait(&self.device).unwrap();
    }

    fn reset_fence(&mut self, fence: Fence) {
        let fence = self.fences.get(&fence.id()).unwrap();
        fence.reset(&self.device).unwrap();
    }

    fn delete_fence(&mut self, fence: Fence) {
        let fence = self.fences.remove(&fence.id()).unwrap();
        unsafe {
            self.device
                .get_device_raw()
                .destroy_fence(*fence.get_fence_raw(), None);
        }
    }

    fn create_semaphore(&mut self) -> Semaphore {
        let semaphore = VulkanSemaphore::new(&self.device).unwrap();
        let semaphore_id = semaphore.id();
        self.semaphores.insert(semaphore_id, semaphore);
        Semaphore::new(semaphore_id)
    }

    fn delete_semaphore(&mut self, semaphore: Semaphore) {
        let semaphore = self.semaphores.remove(&semaphore.id()).unwrap();
        unsafe {
            self.device
                .get_device_raw()
                .destroy_semaphore(*semaphore.get_semaphore_raw(), None);
        }
    }
}
