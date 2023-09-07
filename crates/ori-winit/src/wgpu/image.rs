use std::{collections::HashMap, sync::Arc};

use ori_core::image::{Image, ImageId, WeakImage};
use wgpu::{
    util::DeviceExt, AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Device,
    Extent3d, FilterMode, Queue, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureView, TextureViewDimension,
};

#[derive(Debug)]
pub struct CachedImage {
    weak: WeakImage,
    pub view: TextureView,
    pub sampler: Sampler,
    pub bind_group: BindGroup,
}

#[derive(Debug)]
pub struct ImageCache {
    bind_group_layout: BindGroupLayout,
    fallback_image: Image,
    images: HashMap<ImageId, Arc<CachedImage>>,
}

impl ImageCache {
    pub fn new(device: &Device) -> Self {
        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: Some("ori_image_bind_group_layout"),
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        multisampled: false,
                        view_dimension: TextureViewDimension::D2,
                        sample_type: TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });

        Self {
            bind_group_layout,
            fallback_image: Image::default(),
            images: HashMap::new(),
        }
    }

    pub fn bind_group_layout(&self) -> &BindGroupLayout {
        &self.bind_group_layout
    }

    pub fn clean(&mut self) {
        self.images.retain(|_, image| image.weak.strong_count() > 0);
    }

    pub fn get(&mut self, device: &Device, queue: &Queue, image: &Image) -> Arc<CachedImage> {
        if let Some(image) = self.images.get(&image.id()) {
            return image.clone();
        }

        let texture = device.create_texture_with_data(
            queue,
            &TextureDescriptor {
                label: Some("ori_image"),
                size: Extent3d {
                    width: image.width(),
                    height: image.height(),
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: TextureDimension::D2,
                format: TextureFormat::Rgba8Unorm,
                usage: TextureUsages::COPY_DST | TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            },
            image,
        );

        let view = texture.create_view(&Default::default());

        let filter = if image.filter() {
            FilterMode::Linear
        } else {
            FilterMode::Nearest
        };

        let sampler = device.create_sampler(&SamplerDescriptor {
            label: Some("ori_image_sampler"),
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: filter,
            min_filter: filter,
            ..Default::default()
        });

        let bind_group = device.create_bind_group(&BindGroupDescriptor {
            label: Some("ori_image_bind_group"),
            layout: &self.bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: BindingResource::TextureView(&view),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&sampler),
                },
            ],
        });

        let image = Arc::new(CachedImage {
            weak: image.downgrade(),
            view,
            sampler,
            bind_group,
        });

        self.images.entry(image.weak.id()).or_insert(image).clone()
    }

    pub fn fallback(&mut self, device: &Device, queue: &Queue) -> Arc<CachedImage> {
        self.get(device, queue, &self.fallback_image.clone())
    }
}
