use interfaces::{EventCode, IoFrontend};

use sdl2::event::{Event, WindowEvent};
use sdl2::{pixels::Color, rect::Point, render::Canvas, video::Window, Sdl};
use std::collections::HashMap;
use std::time::{Duration, Instant};

use sdl2::keyboard::Keycode as SdlKeycode;

const MAX_SCREEN_UPDATE_FREQUENCY: u32 = 60; // Herz

pub struct FrontendSdl {
    sdl_context: Sdl,
    canvas: Canvas<Window>,

    custom_keys_mapping: HashMap<EventCode, EventCode>,

    // Logical width (game resolution).
    screen_width: u32,

    last_canvas_update: Instant,
}

impl FrontendSdl {
    pub fn new(
        window_title: &str,
        custom_keys_mapping: HashMap<EventCode, EventCode>,
    ) -> FrontendSdl {
        let sdl_context = sdl2::init().unwrap();

        let start_width = 0;
        let start_height = 0;

        let mut window = sdl_context
            .video()
            .unwrap()
            .window(window_title, start_width, start_height)
            .maximized()
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let mut event_pump = sdl_context.event_pump().unwrap();

        for event in event_pump.poll_iter() {
            if let Event::Window {
                win_event: WindowEvent::SizeChanged(new_width, new_height),
                ..
            } = event
            {
                window
                    .set_size(new_width as u32, new_height as u32)
                    .unwrap();
                break;
            }
        }

        let canvas = window
            .into_canvas()
            .target_texture()
            .present_vsync()
            .build()
            .unwrap();

        FrontendSdl {
            sdl_context: sdl_context,
            canvas: canvas,
            custom_keys_mapping: custom_keys_mapping,
            screen_width: start_width,
            last_canvas_update: Instant::now(),
        }
    }

    // Ugly but necessary, as we can't trivially map an enum to another enum
    //
    fn sdl_to_io_frontend_keycode(sdl_keycode: SdlKeycode) -> EventCode {
        match sdl_keycode {
            SdlKeycode::Backspace => EventCode::KeyBackspace,
            SdlKeycode::Tab => EventCode::KeyTab,
            SdlKeycode::Return => EventCode::KeyReturn,
            SdlKeycode::Escape => EventCode::KeyEscape,
            SdlKeycode::Space => EventCode::KeySpace,
            SdlKeycode::Exclaim => EventCode::KeyExclaim,
            SdlKeycode::Quotedbl => EventCode::KeyQuotedbl,
            SdlKeycode::Hash => EventCode::KeyHash,
            SdlKeycode::Dollar => EventCode::KeyDollar,
            SdlKeycode::Percent => EventCode::KeyPercent,
            SdlKeycode::Ampersand => EventCode::KeyAmpersand,
            SdlKeycode::Quote => EventCode::KeyQuote,
            SdlKeycode::LeftParen => EventCode::KeyLeftParen,
            SdlKeycode::RightParen => EventCode::KeyRightParen,
            SdlKeycode::Asterisk => EventCode::KeyAsterisk,
            SdlKeycode::Plus => EventCode::KeyPlus,
            SdlKeycode::Comma => EventCode::KeyComma,
            SdlKeycode::Minus => EventCode::KeyMinus,
            SdlKeycode::Period => EventCode::KeyPeriod,
            SdlKeycode::Slash => EventCode::KeySlash,
            SdlKeycode::Num0 => EventCode::KeyNum0,
            SdlKeycode::Num1 => EventCode::KeyNum1,
            SdlKeycode::Num2 => EventCode::KeyNum2,
            SdlKeycode::Num3 => EventCode::KeyNum3,
            SdlKeycode::Num4 => EventCode::KeyNum4,
            SdlKeycode::Num5 => EventCode::KeyNum5,
            SdlKeycode::Num6 => EventCode::KeyNum6,
            SdlKeycode::Num7 => EventCode::KeyNum7,
            SdlKeycode::Num8 => EventCode::KeyNum8,
            SdlKeycode::Num9 => EventCode::KeyNum9,
            SdlKeycode::Colon => EventCode::KeyColon,
            SdlKeycode::Semicolon => EventCode::KeySemicolon,
            SdlKeycode::Less => EventCode::KeyLess,
            SdlKeycode::Equals => EventCode::KeyEquals,
            SdlKeycode::Greater => EventCode::KeyGreater,
            SdlKeycode::Question => EventCode::KeyQuestion,
            SdlKeycode::At => EventCode::KeyAt,
            SdlKeycode::LeftBracket => EventCode::KeyLeftBracket,
            SdlKeycode::Backslash => EventCode::KeyBackslash,
            SdlKeycode::RightBracket => EventCode::KeyRightBracket,
            SdlKeycode::Caret => EventCode::KeyCaret,
            SdlKeycode::Underscore => EventCode::KeyUnderscore,
            SdlKeycode::Backquote => EventCode::KeyBackquote,
            SdlKeycode::A => EventCode::KeyA,
            SdlKeycode::B => EventCode::KeyB,
            SdlKeycode::C => EventCode::KeyC,
            SdlKeycode::D => EventCode::KeyD,
            SdlKeycode::E => EventCode::KeyE,
            SdlKeycode::F => EventCode::KeyF,
            SdlKeycode::G => EventCode::KeyG,
            SdlKeycode::H => EventCode::KeyH,
            SdlKeycode::I => EventCode::KeyI,
            SdlKeycode::J => EventCode::KeyJ,
            SdlKeycode::K => EventCode::KeyK,
            SdlKeycode::L => EventCode::KeyL,
            SdlKeycode::M => EventCode::KeyM,
            SdlKeycode::N => EventCode::KeyN,
            SdlKeycode::O => EventCode::KeyO,
            SdlKeycode::P => EventCode::KeyP,
            SdlKeycode::Q => EventCode::KeyQ,
            SdlKeycode::R => EventCode::KeyR,
            SdlKeycode::S => EventCode::KeyS,
            SdlKeycode::T => EventCode::KeyT,
            SdlKeycode::U => EventCode::KeyU,
            SdlKeycode::V => EventCode::KeyV,
            SdlKeycode::W => EventCode::KeyW,
            SdlKeycode::X => EventCode::KeyX,
            SdlKeycode::Y => EventCode::KeyY,
            SdlKeycode::Z => EventCode::KeyZ,
            SdlKeycode::Delete => EventCode::KeyDelete,
            SdlKeycode::CapsLock => EventCode::KeyCapsLock,
            SdlKeycode::F1 => EventCode::KeyF1,
            SdlKeycode::F2 => EventCode::KeyF2,
            SdlKeycode::F3 => EventCode::KeyF3,
            SdlKeycode::F4 => EventCode::KeyF4,
            SdlKeycode::F5 => EventCode::KeyF5,
            SdlKeycode::F6 => EventCode::KeyF6,
            SdlKeycode::F7 => EventCode::KeyF7,
            SdlKeycode::F8 => EventCode::KeyF8,
            SdlKeycode::F9 => EventCode::KeyF9,
            SdlKeycode::F10 => EventCode::KeyF10,
            SdlKeycode::F11 => EventCode::KeyF11,
            SdlKeycode::F12 => EventCode::KeyF12,
            SdlKeycode::PrintScreen => EventCode::KeyPrintScreen,
            SdlKeycode::ScrollLock => EventCode::KeyScrollLock,
            SdlKeycode::Pause => EventCode::KeyPause,
            SdlKeycode::Insert => EventCode::KeyInsert,
            SdlKeycode::Home => EventCode::KeyHome,
            SdlKeycode::PageUp => EventCode::KeyPageUp,
            SdlKeycode::End => EventCode::KeyEnd,
            SdlKeycode::PageDown => EventCode::KeyPageDown,
            SdlKeycode::Right => EventCode::KeyRight,
            SdlKeycode::Left => EventCode::KeyLeft,
            SdlKeycode::Down => EventCode::KeyDown,
            SdlKeycode::Up => EventCode::KeyUp,
            SdlKeycode::NumLockClear => EventCode::KeyNumLockClear,
            SdlKeycode::KpDivide => EventCode::KeyKpDivide,
            SdlKeycode::KpMultiply => EventCode::KeyKpMultiply,
            SdlKeycode::KpMinus => EventCode::KeyKpMinus,
            SdlKeycode::KpPlus => EventCode::KeyKpPlus,
            SdlKeycode::KpEnter => EventCode::KeyKpEnter,
            SdlKeycode::Kp1 => EventCode::KeyKp1,
            SdlKeycode::Kp2 => EventCode::KeyKp2,
            SdlKeycode::Kp3 => EventCode::KeyKp3,
            SdlKeycode::Kp4 => EventCode::KeyKp4,
            SdlKeycode::Kp5 => EventCode::KeyKp5,
            SdlKeycode::Kp6 => EventCode::KeyKp6,
            SdlKeycode::Kp7 => EventCode::KeyKp7,
            SdlKeycode::Kp8 => EventCode::KeyKp8,
            SdlKeycode::Kp9 => EventCode::KeyKp9,
            SdlKeycode::Kp0 => EventCode::KeyKp0,
            SdlKeycode::KpPeriod => EventCode::KeyKpPeriod,
            SdlKeycode::Application => EventCode::KeyApplication,
            SdlKeycode::Power => EventCode::KeyPower,
            SdlKeycode::KpEquals => EventCode::KeyKpEquals,
            SdlKeycode::F13 => EventCode::KeyF13,
            SdlKeycode::F14 => EventCode::KeyF14,
            SdlKeycode::F15 => EventCode::KeyF15,
            SdlKeycode::F16 => EventCode::KeyF16,
            SdlKeycode::F17 => EventCode::KeyF17,
            SdlKeycode::F18 => EventCode::KeyF18,
            SdlKeycode::F19 => EventCode::KeyF19,
            SdlKeycode::F20 => EventCode::KeyF20,
            SdlKeycode::F21 => EventCode::KeyF21,
            SdlKeycode::F22 => EventCode::KeyF22,
            SdlKeycode::F23 => EventCode::KeyF23,
            SdlKeycode::F24 => EventCode::KeyF24,
            SdlKeycode::Execute => EventCode::KeyExecute,
            SdlKeycode::Help => EventCode::KeyHelp,
            SdlKeycode::Menu => EventCode::KeyMenu,
            SdlKeycode::Select => EventCode::KeySelect,
            SdlKeycode::Stop => EventCode::KeyStop,
            SdlKeycode::Again => EventCode::KeyAgain,
            SdlKeycode::Undo => EventCode::KeyUndo,
            SdlKeycode::Cut => EventCode::KeyCut,
            SdlKeycode::Copy => EventCode::KeyCopy,
            SdlKeycode::Paste => EventCode::KeyPaste,
            SdlKeycode::Find => EventCode::KeyFind,
            SdlKeycode::Mute => EventCode::KeyMute,
            SdlKeycode::VolumeUp => EventCode::KeyVolumeUp,
            SdlKeycode::VolumeDown => EventCode::KeyVolumeDown,
            SdlKeycode::KpComma => EventCode::KeyKpComma,
            SdlKeycode::KpEqualsAS400 => EventCode::KeyKpEqualsAS400,
            SdlKeycode::AltErase => EventCode::KeyAltErase,
            SdlKeycode::Sysreq => EventCode::KeySysreq,
            SdlKeycode::Cancel => EventCode::KeyCancel,
            SdlKeycode::Clear => EventCode::KeyClear,
            SdlKeycode::Prior => EventCode::KeyPrior,
            SdlKeycode::Return2 => EventCode::KeyReturn2,
            SdlKeycode::Separator => EventCode::KeySeparator,
            SdlKeycode::Out => EventCode::KeyOut,
            SdlKeycode::Oper => EventCode::KeyOper,
            SdlKeycode::ClearAgain => EventCode::KeyClearAgain,
            SdlKeycode::CrSel => EventCode::KeyCrSel,
            SdlKeycode::ExSel => EventCode::KeyExSel,
            SdlKeycode::Kp00 => EventCode::KeyKp00,
            SdlKeycode::Kp000 => EventCode::KeyKp000,
            SdlKeycode::ThousandsSeparator => EventCode::KeyThousandsSeparator,
            SdlKeycode::DecimalSeparator => EventCode::KeyDecimalSeparator,
            SdlKeycode::CurrencyUnit => EventCode::KeyCurrencyUnit,
            SdlKeycode::CurrencySubUnit => EventCode::KeyCurrencySubUnit,
            SdlKeycode::KpLeftParen => EventCode::KeyKpLeftParen,
            SdlKeycode::KpRightParen => EventCode::KeyKpRightParen,
            SdlKeycode::KpLeftBrace => EventCode::KeyKpLeftBrace,
            SdlKeycode::KpRightBrace => EventCode::KeyKpRightBrace,
            SdlKeycode::KpTab => EventCode::KeyKpTab,
            SdlKeycode::KpBackspace => EventCode::KeyKpBackspace,
            SdlKeycode::KpA => EventCode::KeyKpA,
            SdlKeycode::KpB => EventCode::KeyKpB,
            SdlKeycode::KpC => EventCode::KeyKpC,
            SdlKeycode::KpD => EventCode::KeyKpD,
            SdlKeycode::KpE => EventCode::KeyKpE,
            SdlKeycode::KpF => EventCode::KeyKpF,
            SdlKeycode::KpXor => EventCode::KeyKpXor,
            SdlKeycode::KpPower => EventCode::KeyKpPower,
            SdlKeycode::KpPercent => EventCode::KeyKpPercent,
            SdlKeycode::KpLess => EventCode::KeyKpLess,
            SdlKeycode::KpGreater => EventCode::KeyKpGreater,
            SdlKeycode::KpAmpersand => EventCode::KeyKpAmpersand,
            SdlKeycode::KpDblAmpersand => EventCode::KeyKpDblAmpersand,
            SdlKeycode::KpVerticalBar => EventCode::KeyKpVerticalBar,
            SdlKeycode::KpDblVerticalBar => EventCode::KeyKpDblVerticalBar,
            SdlKeycode::KpColon => EventCode::KeyKpColon,
            SdlKeycode::KpHash => EventCode::KeyKpHash,
            SdlKeycode::KpSpace => EventCode::KeyKpSpace,
            SdlKeycode::KpAt => EventCode::KeyKpAt,
            SdlKeycode::KpExclam => EventCode::KeyKpExclam,
            SdlKeycode::KpMemStore => EventCode::KeyKpMemStore,
            SdlKeycode::KpMemRecall => EventCode::KeyKpMemRecall,
            SdlKeycode::KpMemClear => EventCode::KeyKpMemClear,
            SdlKeycode::KpMemAdd => EventCode::KeyKpMemAdd,
            SdlKeycode::KpMemSubtract => EventCode::KeyKpMemSubtract,
            SdlKeycode::KpMemMultiply => EventCode::KeyKpMemMultiply,
            SdlKeycode::KpMemDivide => EventCode::KeyKpMemDivide,
            SdlKeycode::KpPlusMinus => EventCode::KeyKpPlusMinus,
            SdlKeycode::KpClear => EventCode::KeyKpClear,
            SdlKeycode::KpClearEntry => EventCode::KeyKpClearEntry,
            SdlKeycode::KpBinary => EventCode::KeyKpBinary,
            SdlKeycode::KpOctal => EventCode::KeyKpOctal,
            SdlKeycode::KpDecimal => EventCode::KeyKpDecimal,
            SdlKeycode::KpHexadecimal => EventCode::KeyKpHexadecimal,
            SdlKeycode::LCtrl => EventCode::KeyLCtrl,
            SdlKeycode::LShift => EventCode::KeyLShift,
            SdlKeycode::LAlt => EventCode::KeyLAlt,
            SdlKeycode::LGui => EventCode::KeyLGui,
            SdlKeycode::RCtrl => EventCode::KeyRCtrl,
            SdlKeycode::RShift => EventCode::KeyRShift,
            SdlKeycode::RAlt => EventCode::KeyRAlt,
            SdlKeycode::RGui => EventCode::KeyRGui,
            SdlKeycode::Mode => EventCode::KeyMode,
            SdlKeycode::AudioNext => EventCode::KeyAudioNext,
            SdlKeycode::AudioPrev => EventCode::KeyAudioPrev,
            SdlKeycode::AudioStop => EventCode::KeyAudioStop,
            SdlKeycode::AudioPlay => EventCode::KeyAudioPlay,
            SdlKeycode::AudioMute => EventCode::KeyAudioMute,
            SdlKeycode::MediaSelect => EventCode::KeyMediaSelect,
            SdlKeycode::Www => EventCode::KeyWww,
            SdlKeycode::Mail => EventCode::KeyMail,
            SdlKeycode::Calculator => EventCode::KeyCalculator,
            SdlKeycode::Computer => EventCode::KeyComputer,
            SdlKeycode::AcSearch => EventCode::KeyAcSearch,
            SdlKeycode::AcHome => EventCode::KeyAcHome,
            SdlKeycode::AcBack => EventCode::KeyAcBack,
            SdlKeycode::AcForward => EventCode::KeyAcForward,
            SdlKeycode::AcStop => EventCode::KeyAcStop,
            SdlKeycode::AcRefresh => EventCode::KeyAcRefresh,
            SdlKeycode::AcBookmarks => EventCode::KeyAcBookmarks,
            SdlKeycode::BrightnessDown => EventCode::KeyBrightnessDown,
            SdlKeycode::BrightnessUp => EventCode::KeyBrightnessUp,
            SdlKeycode::DisplaySwitch => EventCode::KeyDisplaySwitch,
            SdlKeycode::KbdIllumToggle => EventCode::KeyKbdIllumToggle,
            SdlKeycode::KbdIllumDown => EventCode::KeyKbdIllumDown,
            SdlKeycode::KbdIllumUp => EventCode::KeyKbdIllumUp,
            SdlKeycode::Eject => EventCode::KeyEject,
            SdlKeycode::Sleep => EventCode::KeySleep,
        }
    }
}

impl IoFrontend for FrontendSdl {
    fn init(&mut self, screen_width: u32, screen_height: u32) {
        let window = self.canvas.window();

        let (window_width, window_height) = window.size();

        let min_scale = f32::min(
            (window_width as f32) / (screen_width as f32).floor(),
            (window_height as f32) / (screen_height as f32).floor(),
        );

        self.canvas.set_scale(min_scale, min_scale).unwrap();

        self.screen_width = screen_width;
    }

    // Capped to MAX_SCREEN_UPDATE_FREQUENCY.
    //
    fn update_screen(&mut self, pixels: &Vec<(u8, u8, u8)>) {
        if self.last_canvas_update.elapsed() >= Duration::new(1, 0) / MAX_SCREEN_UPDATE_FREQUENCY {
            for (y, line) in pixels.chunks(self.screen_width as usize).enumerate() {
                for (x, (r, g, b)) in line.iter().enumerate() {
                    self.canvas.set_draw_color(Color::RGB(*r, *g, *b));

                    self.canvas
                        .draw_point(Point::new(x as i32, y as i32))
                        .unwrap();
                }
            }

            self.canvas.present();
            self.last_canvas_update = Instant::now();
        }
    }

    fn poll_event(&mut self) -> Option<(EventCode, bool)> {
        let mut event_pump = self.sdl_context.event_pump().unwrap();
        let mut key_event = None;

        for event in event_pump.poll_iter() {
            if let Event::KeyDown { keycode, .. } = event {
                if let Some(keycode) = keycode {
                    key_event = Some((FrontendSdl::sdl_to_io_frontend_keycode(keycode), true));
                    break;
                }
            } else if let Event::KeyUp { keycode, .. } = event {
                if let Some(keycode) = keycode {
                    key_event = Some((FrontendSdl::sdl_to_io_frontend_keycode(keycode), false));
                    break;
                }
            } else if let Event::Quit { .. } = event {
                key_event = Some((EventCode::Quit, true));
                break;
            }
        }

        if let Some((event_code, key_pressed)) = key_event {
            if let Some(mapped_key) = self.custom_keys_mapping.get(&event_code) {
                Some((mapped_key.clone(), key_pressed))
            } else {
                Some((event_code, key_pressed))
            }
        } else {
            None
        }
    }

    fn wait_keypress(&mut self) -> interfaces::EventCode {
        let mut event_pump = self.sdl_context.event_pump().unwrap();

        for event in event_pump.wait_iter() {
            if let Event::KeyDown { keycode, .. } = event {
                if let Some(keycode) = keycode {
                    return FrontendSdl::sdl_to_io_frontend_keycode(keycode);
                }
            }
        }

        unreachable!()
    }
}
