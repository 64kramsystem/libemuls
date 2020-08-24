pub const AUDIO_DEVICE_FREQUENCY: u32 = 44100;

pub trait AudioDevice {
    fn play(&mut self);
    fn pause(&mut self);
}
