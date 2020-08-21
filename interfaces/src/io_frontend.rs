use crate::event_code::EventCode;

pub trait IoFrontend {
    fn init(&mut self, screen_width: u32, screen_height: u32);

    fn update_screen(&mut self, pixels: &[(u8, u8, u8)], force_update: bool);

    // speed_factor: 1.0 is the standard; higher numbers shorten the sound, and viceversa.
    //
    fn beep(&mut self, speed_factor: f32);

    // True/false for key pressed/released.
    //
    fn read_event(&mut self, blocking: bool) -> Option<(EventCode, bool)>;
}
