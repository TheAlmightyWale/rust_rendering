pub trait Surface {
    fn set_pixel(&mut self, x: u32, y: u32, color: &Color<u8>);
    fn get_width(&self) -> u32;
    fn get_height(&self) -> u32;
}
