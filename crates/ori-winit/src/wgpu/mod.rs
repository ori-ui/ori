mod image;
mod instance;
mod mesh;
mod quad;
mod render;

pub use instance::*;
pub use render::*;

use image::*;
use mesh::*;
use quad::*;

unsafe fn bytes_of<T>(data: &T) -> &[u8] {
    std::slice::from_raw_parts(data as *const _ as *const u8, std::mem::size_of::<T>())
}

unsafe fn bytes_of_slice<T>(data: &[T]) -> &[u8] {
    std::slice::from_raw_parts(data.as_ptr() as *const u8, std::mem::size_of_val(data))
}
