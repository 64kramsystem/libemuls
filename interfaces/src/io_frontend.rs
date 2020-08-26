use crate::audio::AudioDevice;
use crate::events::EventCode;
use crate::video::Pixel;

/// IoFrontend represent the user-facing interface: audio, video and events.
///
/// The desired trait object is instantiated (eg. SDL, testing, ...) and passed to the platform
/// library, which will use to receive events, and to render audio/video.
///
pub trait IoFrontend {
    fn init(&mut self, screen_width: u32, screen_height: u32);

    /// Requests a screen update to the implementor.
    ///
    /// Implementors can provide screen capping; as a consequence, unless `force_update` is
    /// specified, the function does not guarantee a screen update.
    /// Client platform libraries need to be aware of this; for example, if they have a wait
    /// instruction (e.g. CHIP-8), it needs to send a forced update, because the last frame may
    /// have been discarded.
    ///
    fn update_screen(&mut self, pixels: &[Pixel], force_update: bool);

    /// Retrieve an AudioDevice used for playing a sound wave.
    ///
    /// # Arguments/Return value
    ///
    /// * `generator` - wave generator function; sample_i represent the index of the sample over
    ///   time; it increments monotonically with a step of 1; returns the amplitude.
    ///
    fn audio_device(&mut self, generator: fn(sample_i: u32) -> i16) -> Box<dyn AudioDevice>;

    /// Read an event.
    ///
    /// This function is special, as it must be handled at two different levels: the implementor
    /// must directly handle frontend-related events, e.q. screen resize; the keyboard and quit
    /// events are instead relayed to the platform library.
    ///
    /// # Arguments/return value:
    ///
    /// * `blocking` - true: wait; poll otherwise.
    /// * returns the event (depends on `blocking` and availability); the second member is used
    ///   for keyboard events, and indicates whether they key was pressed (true) or released (false).
    ///
    fn read_event(&mut self, blocking: bool) -> Option<(EventCode, bool)>;
}
