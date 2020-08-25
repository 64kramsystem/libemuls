use interfaces::{audio::AudioDevice, audio::AUDIO_DEVICE_FREQUENCY};
use sdl2::{
    audio::{AudioCallback, AudioSpecDesired},
    AudioSubsystem,
};

struct SimpleCallback {
    generator: fn(sample_i: u32) -> i16,
    sample_i: u32,
}

impl AudioCallback for SimpleCallback {
    type Channel = i16;

    fn callback(&mut self, out: &mut [i16]) {
        for input in out.iter_mut() {
            *input = (self.generator)(self.sample_i);

            // Simplistic. It will break the wave when wrapping around (unless the 2**32 is a
            // multiplier of the period).
            //
            // Current Rust doesn't allow destructuring assignments.
            //
            let (new_i, _) = self.sample_i.overflowing_add(1);
            self.sample_i = new_i;
        }
    }
}

pub(crate) struct AudioDeviceSdl {
    audio_device: sdl2::audio::AudioDevice<SimpleCallback>,
}

impl AudioDeviceSdl {
    pub fn new(
        audio_subsystem: &AudioSubsystem,
        generator: fn(sample_i: u32) -> i16,
    ) -> AudioDeviceSdl {
        let audio_spec = AudioSpecDesired {
            freq: Some(AUDIO_DEVICE_FREQUENCY as i32),
            channels: Some(1),
            samples: None,
        };

        let audio_device = audio_subsystem
            .open_playback(None, &audio_spec, |_spec| SimpleCallback {
                generator,
                sample_i: 0,
            })
            .unwrap();

        AudioDeviceSdl { audio_device }
    }
}

impl AudioDevice for AudioDeviceSdl {
    fn play(&mut self) {
        self.audio_device.resume();
    }

    fn pause(&mut self) {
        self.audio_device.pause();
    }
}
