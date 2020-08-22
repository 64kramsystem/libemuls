use crate::event_code::EventCode;
use crate::pixel::Pixel;

pub trait IoFrontend {
    fn init(&mut self, screen_width: u32, screen_height: u32);

    fn update_screen(&mut self, pixels: &[Pixel], force_update: bool);

    // True/false for key pressed/released.
    //
    fn read_event(&mut self, blocking: bool) -> Option<(EventCode, bool)>;
}
