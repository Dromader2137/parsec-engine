use crate::{
    ecs::{
        system::system,
        world::{fetch::Mut, query::Query},
    },
    graphics::{
        renderer::components::camera::Camera,
        vulkan::{
            VulkanError,
            buffer::{Buffer, BufferUsage},
            descriptor_set::{
                DescriptorPool, DescriptorSet, DescriptorSetBinding,
                DescriptorSetLayout, DescriptorStage, DescriptorType,
            },
            device::Device,
            physical_device::PhysicalDevice,
        },
        window::WindowWrapper,
    },
    math::mat::Matrix4f,
    resources::Resource,
    utils::id_vec::IdVec,
};

pub struct CameraData {
    pub projection_matrix: Matrix4f,
    pub projection_buffer: Buffer,
    pub projection_set: DescriptorSet,
}

impl CameraData {
    pub fn new(
        window: &WindowWrapper,
        physical_device: &PhysicalDevice,
        device: &Device,
        descriptor_pool: &DescriptorPool,
        vfov: f32,
        near: f32,
        far: f32,
    ) -> Result<CameraData, VulkanError> {
        let projection_matrix =
            Matrix4f::perspective(vfov, window.aspect_ratio(), near, far);
        let projection_buffer = Buffer::from_vec(
            physical_device,
            device,
            &[projection_matrix],
            BufferUsage::UNIFORM_BUFFER,
        )
        .unwrap();
        let projection_set_layout =
            DescriptorSetLayout::new(device, vec![DescriptorSetBinding::new(
                0,
                DescriptorType::UNIFORM_BUFFER,
                DescriptorStage::VERTEX,
            )])
            .unwrap();
        let projection_set =
            DescriptorSet::new(device, &projection_set_layout, descriptor_pool)
                .unwrap();
        projection_set
            .bind_buffer(&projection_buffer, device, &projection_set_layout, 0)
            .unwrap();
        Ok(CameraData {
            projection_matrix,
            projection_buffer,
            projection_set,
        })
    }
}

#[system]
fn add_camera_data(
    window: Resource<WindowWrapper>,
    physical_device: Resource<PhysicalDevice>,
    device: Resource<Device>,
    descriptor_pool: Resource<DescriptorPool>,
    mut cameras_data: Resource<IdVec<CameraData>>,
    mut cameras: Query<Mut<Camera>>,
) {
    for (_, camera) in cameras.iter() {
        if camera.data_id.is_none() {
            let camera_data = CameraData::new(
                &window,
                &physical_device,
                &device,
                &descriptor_pool,
                camera.vertical_fov,
                camera.near_clipping_plane,
                camera.far_clipping_plane,
            )
            .unwrap();

            let data_id = cameras_data.push(camera_data);
            camera.data_id = Some(data_id);
        }
    }
}

#[system]
fn update_camera_data(
    window: Resource<WindowWrapper>,
    device: Resource<Device>,
    mut cameras_data: Resource<IdVec<CameraData>>,
    mut cameras: Query<Camera>,
) {
    let aspect_ratio = window.aspect_ratio();
    for (_, camera) in cameras.iter() {
        if camera.data_id.is_none() {
            continue;
        }
        let camera_data =
            cameras_data.get_mut(camera.data_id.unwrap()).unwrap();
        camera_data.projection_matrix = Matrix4f::perspective(
            camera.vertical_fov,
            aspect_ratio,
            camera.near_clipping_plane,
            camera.far_clipping_plane,
        );
        camera_data
            .projection_buffer
            .update(&device, &[camera_data.projection_matrix])
            .unwrap();
    }
}
