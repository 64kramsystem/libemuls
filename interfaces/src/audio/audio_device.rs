pub const AUDIO_DEVICE_FREQUENCY: u32 = 44100;

/// Plays audio; returned by the IoFrontend implementor, and used by the platform library.
///
pub trait AudioDevice {
    fn play(&mut self);
    fn pause(&mut self);
}
