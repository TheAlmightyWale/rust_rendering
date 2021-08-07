#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Pixel {
    r: u8,
    g: u8,
    b: u8,
    a: u8,
}

pub struct PixelBuffer {
    width: u32,
    height: u32,
    pixels: Vec<Pixel>,
}
