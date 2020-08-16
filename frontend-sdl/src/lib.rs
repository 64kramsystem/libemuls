use interfaces::IoFrontend;

use sdl2::{event::Event, pixels::Color, rect::Point, render::Canvas, video::Window};

use interfaces::Keycode as LibemuKeycode;
use sdl2::{keyboard::Keycode as SdlKeycode, EventPump};

// We start from an arbitrary size - it needs to be a sensible size, because it's the size it
// becomes by default when restoring (=opposite of maximizing).
//
const WINDOW_START_WIDTH: u32 = 640;
const WINDOW_START_HEIGHT: u32 = 480;

pub struct FrontendSdl {
    event_pump: EventPump,
    canvas: Canvas<Window>,
}

impl FrontendSdl {
    pub fn new(window_title: &str) -> FrontendSdl {
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

        FrontendSdl { event_pump, canvas }
    }

    // Ugly but necessary, as we can't trivially map an enum to another enum
    //
    fn sdl_to_io_frontend_keycode(sdl_keycode: SdlKeycode) -> LibemuKeycode {
        match sdl_keycode {
            SdlKeycode::Backspace => LibemuKeycode::Backspace,
            SdlKeycode::Tab => LibemuKeycode::Tab,
            SdlKeycode::Return => LibemuKeycode::Return,
            SdlKeycode::Escape => LibemuKeycode::Escape,
            SdlKeycode::Space => LibemuKeycode::Space,
            SdlKeycode::Exclaim => LibemuKeycode::Exclaim,
            SdlKeycode::Quotedbl => LibemuKeycode::Quotedbl,
            SdlKeycode::Hash => LibemuKeycode::Hash,
            SdlKeycode::Dollar => LibemuKeycode::Dollar,
            SdlKeycode::Percent => LibemuKeycode::Percent,
            SdlKeycode::Ampersand => LibemuKeycode::Ampersand,
            SdlKeycode::Quote => LibemuKeycode::Quote,
            SdlKeycode::LeftParen => LibemuKeycode::LeftParen,
            SdlKeycode::RightParen => LibemuKeycode::RightParen,
            SdlKeycode::Asterisk => LibemuKeycode::Asterisk,
            SdlKeycode::Plus => LibemuKeycode::Plus,
            SdlKeycode::Comma => LibemuKeycode::Comma,
            SdlKeycode::Minus => LibemuKeycode::Minus,
            SdlKeycode::Period => LibemuKeycode::Period,
            SdlKeycode::Slash => LibemuKeycode::Slash,
            SdlKeycode::Num0 => LibemuKeycode::Num0,
            SdlKeycode::Num1 => LibemuKeycode::Num1,
            SdlKeycode::Num2 => LibemuKeycode::Num2,
            SdlKeycode::Num3 => LibemuKeycode::Num3,
            SdlKeycode::Num4 => LibemuKeycode::Num4,
            SdlKeycode::Num5 => LibemuKeycode::Num5,
            SdlKeycode::Num6 => LibemuKeycode::Num6,
            SdlKeycode::Num7 => LibemuKeycode::Num7,
            SdlKeycode::Num8 => LibemuKeycode::Num8,
            SdlKeycode::Num9 => LibemuKeycode::Num9,
            SdlKeycode::Colon => LibemuKeycode::Colon,
            SdlKeycode::Semicolon => LibemuKeycode::Semicolon,
            SdlKeycode::Less => LibemuKeycode::Less,
            SdlKeycode::Equals => LibemuKeycode::Equals,
            SdlKeycode::Greater => LibemuKeycode::Greater,
            SdlKeycode::Question => LibemuKeycode::Question,
            SdlKeycode::At => LibemuKeycode::At,
            SdlKeycode::LeftBracket => LibemuKeycode::LeftBracket,
            SdlKeycode::Backslash => LibemuKeycode::Backslash,
            SdlKeycode::RightBracket => LibemuKeycode::RightBracket,
            SdlKeycode::Caret => LibemuKeycode::Caret,
            SdlKeycode::Underscore => LibemuKeycode::Underscore,
            SdlKeycode::Backquote => LibemuKeycode::Backquote,
            SdlKeycode::A => LibemuKeycode::A,
            SdlKeycode::B => LibemuKeycode::B,
            SdlKeycode::C => LibemuKeycode::C,
            SdlKeycode::D => LibemuKeycode::D,
            SdlKeycode::E => LibemuKeycode::E,
            SdlKeycode::F => LibemuKeycode::F,
            SdlKeycode::G => LibemuKeycode::G,
            SdlKeycode::H => LibemuKeycode::H,
            SdlKeycode::I => LibemuKeycode::I,
            SdlKeycode::J => LibemuKeycode::J,
            SdlKeycode::K => LibemuKeycode::K,
            SdlKeycode::L => LibemuKeycode::L,
            SdlKeycode::M => LibemuKeycode::M,
            SdlKeycode::N => LibemuKeycode::N,
            SdlKeycode::O => LibemuKeycode::O,
            SdlKeycode::P => LibemuKeycode::P,
            SdlKeycode::Q => LibemuKeycode::Q,
            SdlKeycode::R => LibemuKeycode::R,
            SdlKeycode::S => LibemuKeycode::S,
            SdlKeycode::T => LibemuKeycode::T,
            SdlKeycode::U => LibemuKeycode::U,
            SdlKeycode::V => LibemuKeycode::V,
            SdlKeycode::W => LibemuKeycode::W,
            SdlKeycode::X => LibemuKeycode::X,
            SdlKeycode::Y => LibemuKeycode::Y,
            SdlKeycode::Z => LibemuKeycode::Z,
            SdlKeycode::Delete => LibemuKeycode::Delete,
            SdlKeycode::CapsLock => LibemuKeycode::CapsLock,
            SdlKeycode::F1 => LibemuKeycode::F1,
            SdlKeycode::F2 => LibemuKeycode::F2,
            SdlKeycode::F3 => LibemuKeycode::F3,
            SdlKeycode::F4 => LibemuKeycode::F4,
            SdlKeycode::F5 => LibemuKeycode::F5,
            SdlKeycode::F6 => LibemuKeycode::F6,
            SdlKeycode::F7 => LibemuKeycode::F7,
            SdlKeycode::F8 => LibemuKeycode::F8,
            SdlKeycode::F9 => LibemuKeycode::F9,
            SdlKeycode::F10 => LibemuKeycode::F10,
            SdlKeycode::F11 => LibemuKeycode::F11,
            SdlKeycode::F12 => LibemuKeycode::F12,
            SdlKeycode::PrintScreen => LibemuKeycode::PrintScreen,
            SdlKeycode::ScrollLock => LibemuKeycode::ScrollLock,
            SdlKeycode::Pause => LibemuKeycode::Pause,
            SdlKeycode::Insert => LibemuKeycode::Insert,
            SdlKeycode::Home => LibemuKeycode::Home,
            SdlKeycode::PageUp => LibemuKeycode::PageUp,
            SdlKeycode::End => LibemuKeycode::End,
            SdlKeycode::PageDown => LibemuKeycode::PageDown,
            SdlKeycode::Right => LibemuKeycode::Right,
            SdlKeycode::Left => LibemuKeycode::Left,
            SdlKeycode::Down => LibemuKeycode::Down,
            SdlKeycode::Up => LibemuKeycode::Up,
            SdlKeycode::NumLockClear => LibemuKeycode::NumLockClear,
            SdlKeycode::KpDivide => LibemuKeycode::KpDivide,
            SdlKeycode::KpMultiply => LibemuKeycode::KpMultiply,
            SdlKeycode::KpMinus => LibemuKeycode::KpMinus,
            SdlKeycode::KpPlus => LibemuKeycode::KpPlus,
            SdlKeycode::KpEnter => LibemuKeycode::KpEnter,
            SdlKeycode::Kp1 => LibemuKeycode::Kp1,
            SdlKeycode::Kp2 => LibemuKeycode::Kp2,
            SdlKeycode::Kp3 => LibemuKeycode::Kp3,
            SdlKeycode::Kp4 => LibemuKeycode::Kp4,
            SdlKeycode::Kp5 => LibemuKeycode::Kp5,
            SdlKeycode::Kp6 => LibemuKeycode::Kp6,
            SdlKeycode::Kp7 => LibemuKeycode::Kp7,
            SdlKeycode::Kp8 => LibemuKeycode::Kp8,
            SdlKeycode::Kp9 => LibemuKeycode::Kp9,
            SdlKeycode::Kp0 => LibemuKeycode::Kp0,
            SdlKeycode::KpPeriod => LibemuKeycode::KpPeriod,
            SdlKeycode::Application => LibemuKeycode::Application,
            SdlKeycode::Power => LibemuKeycode::Power,
            SdlKeycode::KpEquals => LibemuKeycode::KpEquals,
            SdlKeycode::F13 => LibemuKeycode::F13,
            SdlKeycode::F14 => LibemuKeycode::F14,
            SdlKeycode::F15 => LibemuKeycode::F15,
            SdlKeycode::F16 => LibemuKeycode::F16,
            SdlKeycode::F17 => LibemuKeycode::F17,
            SdlKeycode::F18 => LibemuKeycode::F18,
            SdlKeycode::F19 => LibemuKeycode::F19,
            SdlKeycode::F20 => LibemuKeycode::F20,
            SdlKeycode::F21 => LibemuKeycode::F21,
            SdlKeycode::F22 => LibemuKeycode::F22,
            SdlKeycode::F23 => LibemuKeycode::F23,
            SdlKeycode::F24 => LibemuKeycode::F24,
            SdlKeycode::Execute => LibemuKeycode::Execute,
            SdlKeycode::Help => LibemuKeycode::Help,
            SdlKeycode::Menu => LibemuKeycode::Menu,
            SdlKeycode::Select => LibemuKeycode::Select,
            SdlKeycode::Stop => LibemuKeycode::Stop,
            SdlKeycode::Again => LibemuKeycode::Again,
            SdlKeycode::Undo => LibemuKeycode::Undo,
            SdlKeycode::Cut => LibemuKeycode::Cut,
            SdlKeycode::Copy => LibemuKeycode::Copy,
            SdlKeycode::Paste => LibemuKeycode::Paste,
            SdlKeycode::Find => LibemuKeycode::Find,
            SdlKeycode::Mute => LibemuKeycode::Mute,
            SdlKeycode::VolumeUp => LibemuKeycode::VolumeUp,
            SdlKeycode::VolumeDown => LibemuKeycode::VolumeDown,
            SdlKeycode::KpComma => LibemuKeycode::KpComma,
            SdlKeycode::KpEqualsAS400 => LibemuKeycode::KpEqualsAS400,
            SdlKeycode::AltErase => LibemuKeycode::AltErase,
            SdlKeycode::Sysreq => LibemuKeycode::Sysreq,
            SdlKeycode::Cancel => LibemuKeycode::Cancel,
            SdlKeycode::Clear => LibemuKeycode::Clear,
            SdlKeycode::Prior => LibemuKeycode::Prior,
            SdlKeycode::Return2 => LibemuKeycode::Return2,
            SdlKeycode::Separator => LibemuKeycode::Separator,
            SdlKeycode::Out => LibemuKeycode::Out,
            SdlKeycode::Oper => LibemuKeycode::Oper,
            SdlKeycode::ClearAgain => LibemuKeycode::ClearAgain,
            SdlKeycode::CrSel => LibemuKeycode::CrSel,
            SdlKeycode::ExSel => LibemuKeycode::ExSel,
            SdlKeycode::Kp00 => LibemuKeycode::Kp00,
            SdlKeycode::Kp000 => LibemuKeycode::Kp000,
            SdlKeycode::ThousandsSeparator => LibemuKeycode::ThousandsSeparator,
            SdlKeycode::DecimalSeparator => LibemuKeycode::DecimalSeparator,
            SdlKeycode::CurrencyUnit => LibemuKeycode::CurrencyUnit,
            SdlKeycode::CurrencySubUnit => LibemuKeycode::CurrencySubUnit,
            SdlKeycode::KpLeftParen => LibemuKeycode::KpLeftParen,
            SdlKeycode::KpRightParen => LibemuKeycode::KpRightParen,
            SdlKeycode::KpLeftBrace => LibemuKeycode::KpLeftBrace,
            SdlKeycode::KpRightBrace => LibemuKeycode::KpRightBrace,
            SdlKeycode::KpTab => LibemuKeycode::KpTab,
            SdlKeycode::KpBackspace => LibemuKeycode::KpBackspace,
            SdlKeycode::KpA => LibemuKeycode::KpA,
            SdlKeycode::KpB => LibemuKeycode::KpB,
            SdlKeycode::KpC => LibemuKeycode::KpC,
            SdlKeycode::KpD => LibemuKeycode::KpD,
            SdlKeycode::KpE => LibemuKeycode::KpE,
            SdlKeycode::KpF => LibemuKeycode::KpF,
            SdlKeycode::KpXor => LibemuKeycode::KpXor,
            SdlKeycode::KpPower => LibemuKeycode::KpPower,
            SdlKeycode::KpPercent => LibemuKeycode::KpPercent,
            SdlKeycode::KpLess => LibemuKeycode::KpLess,
            SdlKeycode::KpGreater => LibemuKeycode::KpGreater,
            SdlKeycode::KpAmpersand => LibemuKeycode::KpAmpersand,
            SdlKeycode::KpDblAmpersand => LibemuKeycode::KpDblAmpersand,
            SdlKeycode::KpVerticalBar => LibemuKeycode::KpVerticalBar,
            SdlKeycode::KpDblVerticalBar => LibemuKeycode::KpDblVerticalBar,
            SdlKeycode::KpColon => LibemuKeycode::KpColon,
            SdlKeycode::KpHash => LibemuKeycode::KpHash,
            SdlKeycode::KpSpace => LibemuKeycode::KpSpace,
            SdlKeycode::KpAt => LibemuKeycode::KpAt,
            SdlKeycode::KpExclam => LibemuKeycode::KpExclam,
            SdlKeycode::KpMemStore => LibemuKeycode::KpMemStore,
            SdlKeycode::KpMemRecall => LibemuKeycode::KpMemRecall,
            SdlKeycode::KpMemClear => LibemuKeycode::KpMemClear,
            SdlKeycode::KpMemAdd => LibemuKeycode::KpMemAdd,
            SdlKeycode::KpMemSubtract => LibemuKeycode::KpMemSubtract,
            SdlKeycode::KpMemMultiply => LibemuKeycode::KpMemMultiply,
            SdlKeycode::KpMemDivide => LibemuKeycode::KpMemDivide,
            SdlKeycode::KpPlusMinus => LibemuKeycode::KpPlusMinus,
            SdlKeycode::KpClear => LibemuKeycode::KpClear,
            SdlKeycode::KpClearEntry => LibemuKeycode::KpClearEntry,
            SdlKeycode::KpBinary => LibemuKeycode::KpBinary,
            SdlKeycode::KpOctal => LibemuKeycode::KpOctal,
            SdlKeycode::KpDecimal => LibemuKeycode::KpDecimal,
            SdlKeycode::KpHexadecimal => LibemuKeycode::KpHexadecimal,
            SdlKeycode::LCtrl => LibemuKeycode::LCtrl,
            SdlKeycode::LShift => LibemuKeycode::LShift,
            SdlKeycode::LAlt => LibemuKeycode::LAlt,
            SdlKeycode::LGui => LibemuKeycode::LGui,
            SdlKeycode::RCtrl => LibemuKeycode::RCtrl,
            SdlKeycode::RShift => LibemuKeycode::RShift,
            SdlKeycode::RAlt => LibemuKeycode::RAlt,
            SdlKeycode::RGui => LibemuKeycode::RGui,
            SdlKeycode::Mode => LibemuKeycode::Mode,
            SdlKeycode::AudioNext => LibemuKeycode::AudioNext,
            SdlKeycode::AudioPrev => LibemuKeycode::AudioPrev,
            SdlKeycode::AudioStop => LibemuKeycode::AudioStop,
            SdlKeycode::AudioPlay => LibemuKeycode::AudioPlay,
            SdlKeycode::AudioMute => LibemuKeycode::AudioMute,
            SdlKeycode::MediaSelect => LibemuKeycode::MediaSelect,
            SdlKeycode::Www => LibemuKeycode::Www,
            SdlKeycode::Mail => LibemuKeycode::Mail,
            SdlKeycode::Calculator => LibemuKeycode::Calculator,
            SdlKeycode::Computer => LibemuKeycode::Computer,
            SdlKeycode::AcSearch => LibemuKeycode::AcSearch,
            SdlKeycode::AcHome => LibemuKeycode::AcHome,
            SdlKeycode::AcBack => LibemuKeycode::AcBack,
            SdlKeycode::AcForward => LibemuKeycode::AcForward,
            SdlKeycode::AcStop => LibemuKeycode::AcStop,
            SdlKeycode::AcRefresh => LibemuKeycode::AcRefresh,
            SdlKeycode::AcBookmarks => LibemuKeycode::AcBookmarks,
            SdlKeycode::BrightnessDown => LibemuKeycode::BrightnessDown,
            SdlKeycode::BrightnessUp => LibemuKeycode::BrightnessUp,
            SdlKeycode::DisplaySwitch => LibemuKeycode::DisplaySwitch,
            SdlKeycode::KbdIllumToggle => LibemuKeycode::KbdIllumToggle,
            SdlKeycode::KbdIllumDown => LibemuKeycode::KbdIllumDown,
            SdlKeycode::KbdIllumUp => LibemuKeycode::KbdIllumUp,
            SdlKeycode::Eject => LibemuKeycode::Eject,
            SdlKeycode::Sleep => LibemuKeycode::Sleep,
        }
    }
}

impl IoFrontend for FrontendSdl {
    fn init(&mut self, screen_width: u32, screen_height: u32) {
        let window = self.canvas.window_mut();

        window.set_size(screen_width, screen_height).unwrap();
    }

    fn draw_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8) {
        self.canvas.set_draw_color(Color::RGB(r, g, b));

        self.canvas
            .draw_point(Point::new(x as i32, y as i32))
            .unwrap();
    }

    fn update_screen(&mut self) {
        self.canvas.present();
    }

    fn read_key_event(&mut self, blocking: bool) -> Option<(LibemuKeycode, bool)> {
        loop {
            let event = if blocking {
                Some(self.event_pump.wait_event())
            } else {
                self.event_pump.poll_event()
            };

            match event {
                Some(Event::KeyDown {
                    keycode: Some(keycode),
                    ..
                }) => {
                    return Some((FrontendSdl::sdl_to_io_frontend_keycode(keycode), true));
                }
                Some(Event::KeyUp {
                    keycode: Some(keycode),
                    ..
                }) => {
                    return Some((FrontendSdl::sdl_to_io_frontend_keycode(keycode), false));
                }
                None => {
                    // This happens only for non-blocking events.
                    //
                    return None;
                }
                _ => {
                    // Entirely ignore all the other events.
                }
            }
        }
    }
}
