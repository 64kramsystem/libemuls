#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(non_snake_case)]
#![allow(dead_code)]

#[macro_use]
extern crate maplit;

extern crate rand;

use sdl2::{
    audio::{AudioCallback, AudioSpecDesired},
    AudioSubsystem,
};
use std::f64::consts::PI;
use std::thread;
use std::{collections::HashMap, time::Duration};

const AMPLITUDE: i16 = i16::MAX; // Volume
const DEVICE_FREQUENCY: u32 = 44100; // in Herz

struct SimpleCallback {
    sample_i: u32,
    tone_frequency: f64,
}

// Sine wave; source: https://stackoverflow.com/a/10111570
//
fn with_callback(
    audio_subsystem: &AudioSubsystem,
    desired_spec: &AudioSpecDesired,
    tone_frequency: f64,
) -> sdl2::audio::AudioDevice<SimpleCallback> {
    impl AudioCallback for SimpleCallback {
        type Channel = i16;

        fn callback(&mut self, out: &mut [i16]) {
            let period = DEVICE_FREQUENCY as f64 / self.tone_frequency;

            for input in out.iter_mut() {
                let period_number = self.sample_i as f64 / period;
                let scale_factor = (period_number * 2.0 * PI).sin();
                let sample = (AMPLITUDE as f64 * scale_factor) as i16;

                *input = sample;

                self.sample_i += 1;
            }
        }
    }

    let audio_device = audio_subsystem
        .open_playback(None, desired_spec, |_spec| SimpleCallback {
            sample_i: 0,
            tone_frequency,
        })
        .unwrap();

    audio_device.resume();

    audio_device
}

// Square wave; source: https://git.io/JJNRc
//
fn with_push(
    audio_subsystem: &AudioSubsystem,
    desired_spec: &AudioSpecDesired,
    tone_frequency: f64,
    duration: u64,
) -> sdl2::audio::AudioQueue<i16> {
    let period = DEVICE_FREQUENCY as f64 / tone_frequency;
    let wave_samples_count = DEVICE_FREQUENCY * duration as u32 / 1000; // approx.

    let wave_samples = (0..wave_samples_count)
        .map(|sample_i| {
            let period_number_1 = sample_i / period as u32;
            let scale_sign_1 = if period_number_1 % 2 == 0 { 1 } else { -1 };
            let sample_value_1 = (AMPLITUDE * scale_sign_1) as i16;

            let period_number_2 = sample_i as f64 / period;
            let scale_sign_2 = (period_number_2 * PI).sin().signum();

            (AMPLITUDE as f64 * scale_sign_2) as i16
        })
        .collect::<Vec<i16>>();

    let audio_queue = audio_subsystem
        .open_queue::<i16, _>(None, &desired_spec)
        .unwrap();

    audio_queue.queue(&wave_samples);

    audio_queue.resume();

    audio_queue
}

fn main() {
    let TONE_FREQUENCIES = hashmap! {
        "D" => 293.665,
        "E" => 329.628,
        "F" => 349.228,
        "G" => 391.995,
        "A" => 440.000,
        "B" => 493.883,
        "c" => 554.365,
        "d" => 587.330,
        "PC_SPEAKER" => 750.0,
    };

    let sdl_context = sdl2::init().unwrap();
    let audio_subsystem = sdl_context.audio().unwrap();
    let desired_spec = AudioSpecDesired {
        freq: Some(DEVICE_FREQUENCY as i32),
        channels: Some(1),
        samples: None,
    };

    // let audio_device = with_callback(
    //     &audio_subsystem,
    //     &desired_spec,
    //     TONE_FREQUENCIES["PC_SPEAKER"],
    // );

    // The queue will continue until it's closed (/dropped).
    //
    // thread::sleep(Duration::from_millis(400));
    // audio_device.close_and_get_callback();

    // thread::sleep(Duration::from_millis(400));

    let audio_queue = with_push(
        &audio_subsystem,
        &desired_spec,
        TONE_FREQUENCIES["PC_SPEAKER"],
        400,
    );

    // In this case, the queue will stop as soon as the samples are finished.
    //
    std::thread::sleep(Duration::from_millis(1200));
}
