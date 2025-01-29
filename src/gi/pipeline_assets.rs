use std::path::Path;

use bevy::asset::io::AssetSourceId;
use bevy::asset::AssetPath;
use bevy::prelude::*;
use bevy::render::render_resource::{StorageBuffer, UniformBuffer};
use bevy::render::renderer::{RenderDevice, RenderQueue};
use bevy::render::Extract;
use rand::Rng;

use crate::gi::constants::GI_SCREEN_PROBE_SIZE;
use crate::gi::resource::ComputedTargetSizes;
use crate::gi::types::{LightOccluder2D, OmniLightSource2D, SkylightLight2D, SkylightMask2D};
use crate::gi::types_gpu::{
    GpuCameraParams,
    GpuLightOccluder2D,
    GpuLightOccluderBuffer,
    GpuLightPassParams,
    GpuLightSourceBuffer,
    GpuOmniLightSource,
    GpuProbeDataBuffer,
    GpuSkylightMaskBuffer,
    GpuSkylightMaskData,
};
use crate::prelude::BevyMagicLight2DSettings;
use crate::FloorCamera;

#[rustfmt::skip]
#[derive(Default, Resource)]
pub(crate) struct EmbeddedShaderDependencies {
    loaded_shaders: Vec<Handle<Shader>>,
}

#[rustfmt::skip]
pub(crate) fn system_load_embedded_shader_dependencies(
    mut embedded_shader_deps: ResMut<EmbeddedShaderDependencies>,
    asset_server: Res<AssetServer>,
) {
    embedded_shader_deps.loaded_shaders.push(load_embedded_shader(&asset_server, "gi_attenuation.wgsl"));
    embedded_shader_deps.loaded_shaders.push(load_embedded_shader(&asset_server, "gi_camera.wgsl"));
    embedded_shader_deps.loaded_shaders.push(load_embedded_shader(&asset_server, "gi_halton.wgsl"));
    embedded_shader_deps.loaded_shaders.push(load_embedded_shader(&asset_server, "gi_math.wgsl"));
    embedded_shader_deps.loaded_shaders.push(load_embedded_shader(&asset_server, "gi_post_processing.wgsl"));
    embedded_shader_deps.loaded_shaders.push(load_embedded_shader(&asset_server, "gi_raymarch.wgsl"));
    embedded_shader_deps.loaded_shaders.push(load_embedded_shader(&asset_server, "gi_types.wgsl"));
}

pub(crate) fn load_embedded_shader(asset_server: &AssetServer, shader_file: &str)
    -> Handle<Shader>
{
    let source = AssetSourceId::from("embedded");
    let path = Path::new("bevy_magic_light_2d").join("gi/shaders/");
    asset_server.load(AssetPath::from_path(&path.join(shader_file)).with_source(&source))
}

#[rustfmt::skip]
#[derive(Default, Resource)]
pub struct LightPassPipelineAssets {
    pub camera_params:     UniformBuffer<GpuCameraParams>,
    pub light_pass_params: UniformBuffer<GpuLightPassParams>,
    pub light_sources:     StorageBuffer<GpuLightSourceBuffer>,
    pub light_occluders:   StorageBuffer<GpuLightOccluderBuffer>,
    pub probes:            StorageBuffer<GpuProbeDataBuffer>,
    pub skylight_masks:    StorageBuffer<GpuSkylightMaskBuffer>,
}

impl LightPassPipelineAssets
{
    pub fn write_buffer(&mut self, device: &RenderDevice, queue: &RenderQueue)
    {
        self.light_sources.write_buffer(device, queue);
        self.light_occluders.write_buffer(device, queue);
        self.camera_params.write_buffer(device, queue);
        self.light_pass_params.write_buffer(device, queue);
        self.probes.write_buffer(device, queue);
        self.skylight_masks.write_buffer(device, queue);
    }
}

#[rustfmt::skip]
pub fn system_prepare_pipeline_assets(
    render_device:         Res<RenderDevice>,
    render_queue:          Res<RenderQueue>,
    mut gi_compute_assets: ResMut<LightPassPipelineAssets>,
) {
    gi_compute_assets.write_buffer(&render_device, &render_queue);
}

#[rustfmt::skip]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn system_extract_pipeline_assets(
    res_light_settings:         Extract<Res<BevyMagicLight2DSettings>>,
    res_target_sizes:           Extract<Res<ComputedTargetSizes>>,

    query_lights:               Extract<Query<(&GlobalTransform, &OmniLightSource2D, &InheritedVisibility, &ViewVisibility)>>,
    query_occluders:            Extract<Query<(&LightOccluder2D, &GlobalTransform, &Transform, &InheritedVisibility, &ViewVisibility)>>,
    query_camera:               Extract<Query<(&Camera, &GlobalTransform), With<FloorCamera>>>,
    query_masks:                Extract<Query<(&GlobalTransform, &SkylightMask2D)>>,
    query_skylight_light:       Extract<Query<&SkylightLight2D>>,

    mut gpu_target_sizes:       ResMut<ComputedTargetSizes>,
    mut gpu_pipeline_assets:    ResMut<LightPassPipelineAssets>,
    mut gpu_frame_counter:      Local<i32>,
) {
    let light_pass_config = &res_light_settings.light_pass_params;

    *gpu_target_sizes = **res_target_sizes;

    {
        let light_sources = gpu_pipeline_assets.light_sources.get_mut();
        let mut rng = rand::rng();
        light_sources.count = 0;
        light_sources.data.clear();
        for (transform, light_source, hviz, vviz) in query_lights.iter() {
            if hviz.get() && vviz.get() {
                light_sources.count += 1;
                light_sources.data.push(GpuOmniLightSource::new(
                    OmniLightSource2D {
                        intensity: light_source.intensity
                            + rng.random_range(-1.0..1.0) * light_source.jitter_intensity,
                        ..*light_source
                    },
                    Vec2::new(
                        transform.translation().x
                            + rng.random_range(-1.0..1.0) * light_source.jitter_translation,
                        transform.translation().y
                            + rng.random_range(-1.0..1.0) * light_source.jitter_translation,
                    ),
                ));
            }
        }
    }

    {
        let light_occluders = gpu_pipeline_assets.light_occluders.get_mut();
        light_occluders.count = 0;
        light_occluders.data.clear();
        for (occluder, global_transform, transform, hviz, vviz) in query_occluders.iter() {
            if hviz.get() && vviz.get() {
                light_occluders.count += 1;
                light_occluders.data.push(GpuLightOccluder2D {
                    center: global_transform.translation().xy(),
                    rotation: transform.rotation.inverse().into(),
                    h_extent: occluder.h_size,
                });
            }
        }
    }

    {
        let skylight_masks = gpu_pipeline_assets.skylight_masks.get_mut();
        skylight_masks.count = 0;
        skylight_masks.data.clear();
        for (transform, mask) in query_masks.iter() {
            skylight_masks.count += 1;
            skylight_masks.data.push(GpuSkylightMaskData::new(
                transform.translation().truncate(),
                mask.h_size,
            ));
        }
    }

    {
        if let Ok((camera, camera_global_transform)) = query_camera.get_single() {
            let camera_params = gpu_pipeline_assets.camera_params.get_mut();
            let projection = camera.clip_from_view();
            let inverse_projection = projection.inverse();
            let view = camera_global_transform.compute_matrix();
            let inverse_view = view.inverse();

            camera_params.view_proj = projection * inverse_view;
            camera_params.inverse_view_proj = view * inverse_projection;
            camera_params.screen_size = Vec2::new(
                gpu_target_sizes.primary_target_size.x,
                gpu_target_sizes.primary_target_size.y,
            );
            camera_params.screen_size_inv = Vec2::new(
                1.0 / gpu_target_sizes.primary_target_size.x,
                1.0 / gpu_target_sizes.primary_target_size.y,
            );

            let scale = 2.0;
            camera_params.sdf_scale     = Vec2::splat(scale);
            camera_params.inv_sdf_scale = Vec2::splat(1. / scale);

            let probes = gpu_pipeline_assets.probes.get_mut();
            probes.data[*gpu_frame_counter as usize].camera_pose =
                camera_global_transform.translation().truncate();
        } else {
            warn!("Failed to get camera");
            let probes = gpu_pipeline_assets.probes.get_mut();
            probes.data[*gpu_frame_counter as usize].camera_pose = Vec2::ZERO;
        }
    }

    {
        let light_pass_params = gpu_pipeline_assets.light_pass_params.get_mut();
        light_pass_params.frame_counter = *gpu_frame_counter;
        light_pass_params.probe_size = GI_SCREEN_PROBE_SIZE;
        light_pass_params.probe_atlas_cols            = gpu_target_sizes.probe_grid_isize.x;
        light_pass_params.probe_atlas_rows            = gpu_target_sizes.probe_grid_isize.y;
        light_pass_params.reservoir_size              = light_pass_config.reservoir_size;
        light_pass_params.smooth_kernel_size_h        = light_pass_config.smooth_kernel_size.0;
        light_pass_params.smooth_kernel_size_w        = light_pass_config.smooth_kernel_size.1;
        light_pass_params.direct_light_contrib        = light_pass_config.direct_light_contrib;
        light_pass_params.indirect_light_contrib      = light_pass_config.indirect_light_contrib;
        light_pass_params.indirect_rays_radius_factor = light_pass_config.indirect_rays_radius_factor;
        light_pass_params.indirect_rays_per_sample    = light_pass_config.indirect_rays_per_sample;
    }

    {
        let light_pass_params = gpu_pipeline_assets.light_pass_params.get_mut();
        light_pass_params.skylight_color = Vec3::splat(0.0);
        for new_gi_state in query_skylight_light.iter() {
            let srgba = new_gi_state.color.to_srgba();
            light_pass_params.skylight_color.x += srgba.red * new_gi_state.intensity;
            light_pass_params.skylight_color.y += srgba.green * new_gi_state.intensity;
            light_pass_params.skylight_color.z += srgba.blue * new_gi_state.intensity;
        }
    }

    *gpu_frame_counter = (*gpu_frame_counter + 1) % (GI_SCREEN_PROBE_SIZE * GI_SCREEN_PROBE_SIZE);
}
