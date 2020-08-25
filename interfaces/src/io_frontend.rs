use crate::audio::AudioDevice;
use crate::events::EventCode;
use crate::video::Pixel;

pub trait IoFrontend {
    fn init(&mut self, screen_width: u32, screen_height: u32);

    fn update_screen(&mut self, pixels: &[Pixel], force_update: bool);

    fn audio_device(&mut self, generator: fn(sample_i: u32) -> i16) -> Box<dyn AudioDevice>;

    // True/false for key pressed/released.
    //
    fn read_event(&mut self, blocking: bool) -> Option<(EventCode, bool)>;
}
