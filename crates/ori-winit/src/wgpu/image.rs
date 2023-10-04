use std::{collections::HashMap, ops::Deref, sync::Arc};

use ori_core::image::{Image, ImageId, Texture, WeakImage};
use wgpu::{
    util::DeviceExt, AddressMode, BindGroup, BindGroupDescriptor, BindGroupEntry, BindGroupLayout,
    BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingResource, BindingType, Device,
    Extent3d, FilterMode, Queue, Sampler, SamplerBindingType, SamplerDescriptor, ShaderStages,
    TextureDescriptor, TextureDimension, TextureFormat, TextureSampleType, TextureUsages,
    TextureView, TextureViewDimension,
};

#[derive(Debug)]
pub struct CachedImage {
    weak: Option<WeakImage>,
    pub view: TextureView,
    pub sampler: Sampler,
    pub bind_group: BindGroup,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum ImageCacheKey {
    Image(ImageId),
    Texture(usize),
}

#[derive(Debug)]
pub struct ImageCache {
    bind_group_layout: BindGroupLayout,
    fallback_image: Image,
    images: HashMap<ImageCacheKey, Arc<CachedImage>>,
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
        self.images.retain(|_, image| match image.weak {
            Some(ref weak) => weak.strong_count() > 0,
            None => true,
        });
    }

    pub fn get(&mut self, device: &Device, queue: &Queue, texture: &Texture) -> Arc<CachedImage> {
        let id = match texture {
            Texture::Image(image) => ImageCacheKey::Image(image.id()),
            Texture::Wgpu(texture) => ImageCacheKey::Texture(texture.deref() as *const _ as usize),
        };

        if let Some(image) = self.images.get(&id) {
            return image.clone();
        }

        let filter = match texture {
            Texture::Image(image) if image.filter() => FilterMode::Linear,
            Texture::Image(_) => FilterMode::Nearest,
            Texture::Wgpu(_) => FilterMode::Linear,
        };

        let weak = match texture {
            Texture::Image(image) => Some(image.downgrade()),
            Texture::Wgpu(_) => None,
        };

        let texture = match texture {
            Texture::Image(image) => device.create_texture_with_data(
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
            ),
            Texture::Wgpu(_) => todo!(),
        };

        let view = texture.create_view(&Default::default());

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
            weak,
            view,
            sampler,
            bind_group,
        });

        self.images.entry(id).or_insert(image).clone()
    }

    pub fn fallback(&mut self, device: &Device, queue: &Queue) -> Arc<CachedImage> {
        let texture = Texture::Image(self.fallback_image.clone());
        self.get(device, queue, &texture)
    }
}
