use interfaces::{EventCode, IoFrontend};

use sdl2::{event::Event, pixels::Color, rect::Point, render::Canvas, video::Window};
use std::collections::HashMap;

use sdl2::{keyboard::Keycode as SdlKeycode, EventPump};

// We start from an arbitrary size - it needs to be a sensible size, because it's the size it
// becomes by default when restoring (=opposite of maximizing).
//
const WINDOW_START_WIDTH: u32 = 640;
const WINDOW_START_HEIGHT: u32 = 480;

pub struct FrontendSdl {
    event_pump: EventPump,
    canvas: Canvas<Window>,

    custom_keys_mapping: HashMap<EventCode, EventCode>,

    screen_width: u32,
}

impl FrontendSdl {
    pub fn new(
        window_title: &str,
        custom_keys_mapping: HashMap<EventCode, EventCode>,
    ) -> FrontendSdl {
        let sdl_context = sdl2::init().unwrap();

        let window = sdl_context
            .video()
            .unwrap()
            .window(window_title, WINDOW_START_WIDTH, WINDOW_START_HEIGHT)
            .position_centered()
            .build()
            .unwrap();

        let canvas = window.into_canvas().present_vsync().build().unwrap();

        let event_pump = sdl_context.event_pump().unwrap();

        FrontendSdl {
            event_pump,
            canvas,
            custom_keys_mapping,
            screen_width: WINDOW_START_WIDTH,
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
        self.screen_width = screen_width;

        let window = self.canvas.window_mut();

        window.set_size(screen_width, screen_height).unwrap();
    }

    fn update_screen(&mut self, pixels: &[(u8, u8, u8)]) {
        for (y, line) in pixels.chunks(self.screen_width as usize).enumerate() {
            for (x, (r, g, b)) in line.iter().enumerate() {
                self.canvas.set_draw_color(Color::RGB(*r, *g, *b));

                self.canvas
                    .draw_point(Point::new(x as i32, y as i32))
                    .unwrap();
            }
        }

        self.canvas.present();
    }

    fn read_event(&mut self, blocking: bool) -> Option<(EventCode, bool)> {
        loop {
            let event = if blocking {
                Some(self.event_pump.wait_event())
            } else {
                self.event_pump.poll_event()
            };

            if let Some(Event::KeyDown { keycode, .. }) = event {
                if let Some(keycode) = keycode {
                    let key_code = FrontendSdl::sdl_to_io_frontend_keycode(keycode);

                    // The unwrap() alternative is cool, but doesn't give real gains.
                    //
                    return if let Some(mapped_key) = self.custom_keys_mapping.get(&key_code) {
                        Some((mapped_key.clone(), true))
                    } else {
                        Some((key_code, true))
                    };
                }
            } else if let Some(Event::KeyUp { keycode, .. }) = event {
                if let Some(keycode) = keycode {
                    let key_code = FrontendSdl::sdl_to_io_frontend_keycode(keycode);

                    return if let Some(mapped_key) = self.custom_keys_mapping.get(&key_code) {
                        Some((mapped_key.clone(), false))
                    } else {
                        Some((key_code, false))
                    };
                }
            } else if let Some(Event::Quit { .. }) = event {
                return Some((EventCode::Quit, true));
            } else if let None = event {
                // This happens only for non-blocking events.
                //
                return None;
            }
        }
    }
}
