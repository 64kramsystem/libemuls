use interfaces::IoFrontend;

use sdl2::{event::Event, pixels::Color, rect::Point, render::Canvas, video::Window};

use interfaces::Keycode as LibemuKeycode;
use sdl2::{keyboard::Keycode as SdlKeycode, EventPump};

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
            .window(window_title, 0, 0)
            .position_centered()
            .opengl()
            .build()
            .unwrap();

        let canvas = window
            .into_canvas()
            .target_texture()
            .present_vsync()
            .build()
            .unwrap();

        let event_pump = sdl_context.event_pump().unwrap();

        FrontendSdl { event_pump, canvas }
    }

    // Ugly but necessary, as we can't trivially map an enum to another enum
    //
    fn sdl_to_io_frontend_keycode(sdl_keycode: SdlKeycode, key_pressed: bool) -> LibemuKeycode {
        match sdl_keycode {
            SdlKeycode::Backspace => LibemuKeycode::Backspace(key_pressed),
            SdlKeycode::Tab => LibemuKeycode::Tab(key_pressed),
            SdlKeycode::Return => LibemuKeycode::Return(key_pressed),
            SdlKeycode::Escape => LibemuKeycode::Escape(key_pressed),
            SdlKeycode::Space => LibemuKeycode::Space(key_pressed),
            SdlKeycode::Exclaim => LibemuKeycode::Exclaim(key_pressed),
            SdlKeycode::Quotedbl => LibemuKeycode::Quotedbl(key_pressed),
            SdlKeycode::Hash => LibemuKeycode::Hash(key_pressed),
            SdlKeycode::Dollar => LibemuKeycode::Dollar(key_pressed),
            SdlKeycode::Percent => LibemuKeycode::Percent(key_pressed),
            SdlKeycode::Ampersand => LibemuKeycode::Ampersand(key_pressed),
            SdlKeycode::Quote => LibemuKeycode::Quote(key_pressed),
            SdlKeycode::LeftParen => LibemuKeycode::LeftParen(key_pressed),
            SdlKeycode::RightParen => LibemuKeycode::RightParen(key_pressed),
            SdlKeycode::Asterisk => LibemuKeycode::Asterisk(key_pressed),
            SdlKeycode::Plus => LibemuKeycode::Plus(key_pressed),
            SdlKeycode::Comma => LibemuKeycode::Comma(key_pressed),
            SdlKeycode::Minus => LibemuKeycode::Minus(key_pressed),
            SdlKeycode::Period => LibemuKeycode::Period(key_pressed),
            SdlKeycode::Slash => LibemuKeycode::Slash(key_pressed),
            SdlKeycode::Num0 => LibemuKeycode::Num0(key_pressed),
            SdlKeycode::Num1 => LibemuKeycode::Num1(key_pressed),
            SdlKeycode::Num2 => LibemuKeycode::Num2(key_pressed),
            SdlKeycode::Num3 => LibemuKeycode::Num3(key_pressed),
            SdlKeycode::Num4 => LibemuKeycode::Num4(key_pressed),
            SdlKeycode::Num5 => LibemuKeycode::Num5(key_pressed),
            SdlKeycode::Num6 => LibemuKeycode::Num6(key_pressed),
            SdlKeycode::Num7 => LibemuKeycode::Num7(key_pressed),
            SdlKeycode::Num8 => LibemuKeycode::Num8(key_pressed),
            SdlKeycode::Num9 => LibemuKeycode::Num9(key_pressed),
            SdlKeycode::Colon => LibemuKeycode::Colon(key_pressed),
            SdlKeycode::Semicolon => LibemuKeycode::Semicolon(key_pressed),
            SdlKeycode::Less => LibemuKeycode::Less(key_pressed),
            SdlKeycode::Equals => LibemuKeycode::Equals(key_pressed),
            SdlKeycode::Greater => LibemuKeycode::Greater(key_pressed),
            SdlKeycode::Question => LibemuKeycode::Question(key_pressed),
            SdlKeycode::At => LibemuKeycode::At(key_pressed),
            SdlKeycode::LeftBracket => LibemuKeycode::LeftBracket(key_pressed),
            SdlKeycode::Backslash => LibemuKeycode::Backslash(key_pressed),
            SdlKeycode::RightBracket => LibemuKeycode::RightBracket(key_pressed),
            SdlKeycode::Caret => LibemuKeycode::Caret(key_pressed),
            SdlKeycode::Underscore => LibemuKeycode::Underscore(key_pressed),
            SdlKeycode::Backquote => LibemuKeycode::Backquote(key_pressed),
            SdlKeycode::A => LibemuKeycode::A(key_pressed),
            SdlKeycode::B => LibemuKeycode::B(key_pressed),
            SdlKeycode::C => LibemuKeycode::C(key_pressed),
            SdlKeycode::D => LibemuKeycode::D(key_pressed),
            SdlKeycode::E => LibemuKeycode::E(key_pressed),
            SdlKeycode::F => LibemuKeycode::F(key_pressed),
            SdlKeycode::G => LibemuKeycode::G(key_pressed),
            SdlKeycode::H => LibemuKeycode::H(key_pressed),
            SdlKeycode::I => LibemuKeycode::I(key_pressed),
            SdlKeycode::J => LibemuKeycode::J(key_pressed),
            SdlKeycode::K => LibemuKeycode::K(key_pressed),
            SdlKeycode::L => LibemuKeycode::L(key_pressed),
            SdlKeycode::M => LibemuKeycode::M(key_pressed),
            SdlKeycode::N => LibemuKeycode::N(key_pressed),
            SdlKeycode::O => LibemuKeycode::O(key_pressed),
            SdlKeycode::P => LibemuKeycode::P(key_pressed),
            SdlKeycode::Q => LibemuKeycode::Q(key_pressed),
            SdlKeycode::R => LibemuKeycode::R(key_pressed),
            SdlKeycode::S => LibemuKeycode::S(key_pressed),
            SdlKeycode::T => LibemuKeycode::T(key_pressed),
            SdlKeycode::U => LibemuKeycode::U(key_pressed),
            SdlKeycode::V => LibemuKeycode::V(key_pressed),
            SdlKeycode::W => LibemuKeycode::W(key_pressed),
            SdlKeycode::X => LibemuKeycode::X(key_pressed),
            SdlKeycode::Y => LibemuKeycode::Y(key_pressed),
            SdlKeycode::Z => LibemuKeycode::Z(key_pressed),
            SdlKeycode::Delete => LibemuKeycode::Delete(key_pressed),
            SdlKeycode::CapsLock => LibemuKeycode::CapsLock(key_pressed),
            SdlKeycode::F1 => LibemuKeycode::F1(key_pressed),
            SdlKeycode::F2 => LibemuKeycode::F2(key_pressed),
            SdlKeycode::F3 => LibemuKeycode::F3(key_pressed),
            SdlKeycode::F4 => LibemuKeycode::F4(key_pressed),
            SdlKeycode::F5 => LibemuKeycode::F5(key_pressed),
            SdlKeycode::F6 => LibemuKeycode::F6(key_pressed),
            SdlKeycode::F7 => LibemuKeycode::F7(key_pressed),
            SdlKeycode::F8 => LibemuKeycode::F8(key_pressed),
            SdlKeycode::F9 => LibemuKeycode::F9(key_pressed),
            SdlKeycode::F10 => LibemuKeycode::F10(key_pressed),
            SdlKeycode::F11 => LibemuKeycode::F11(key_pressed),
            SdlKeycode::F12 => LibemuKeycode::F12(key_pressed),
            SdlKeycode::PrintScreen => LibemuKeycode::PrintScreen(key_pressed),
            SdlKeycode::ScrollLock => LibemuKeycode::ScrollLock(key_pressed),
            SdlKeycode::Pause => LibemuKeycode::Pause(key_pressed),
            SdlKeycode::Insert => LibemuKeycode::Insert(key_pressed),
            SdlKeycode::Home => LibemuKeycode::Home(key_pressed),
            SdlKeycode::PageUp => LibemuKeycode::PageUp(key_pressed),
            SdlKeycode::End => LibemuKeycode::End(key_pressed),
            SdlKeycode::PageDown => LibemuKeycode::PageDown(key_pressed),
            SdlKeycode::Right => LibemuKeycode::Right(key_pressed),
            SdlKeycode::Left => LibemuKeycode::Left(key_pressed),
            SdlKeycode::Down => LibemuKeycode::Down(key_pressed),
            SdlKeycode::Up => LibemuKeycode::Up(key_pressed),
            SdlKeycode::NumLockClear => LibemuKeycode::NumLockClear(key_pressed),
            SdlKeycode::KpDivide => LibemuKeycode::KpDivide(key_pressed),
            SdlKeycode::KpMultiply => LibemuKeycode::KpMultiply(key_pressed),
            SdlKeycode::KpMinus => LibemuKeycode::KpMinus(key_pressed),
            SdlKeycode::KpPlus => LibemuKeycode::KpPlus(key_pressed),
            SdlKeycode::KpEnter => LibemuKeycode::KpEnter(key_pressed),
            SdlKeycode::Kp1 => LibemuKeycode::Kp1(key_pressed),
            SdlKeycode::Kp2 => LibemuKeycode::Kp2(key_pressed),
            SdlKeycode::Kp3 => LibemuKeycode::Kp3(key_pressed),
            SdlKeycode::Kp4 => LibemuKeycode::Kp4(key_pressed),
            SdlKeycode::Kp5 => LibemuKeycode::Kp5(key_pressed),
            SdlKeycode::Kp6 => LibemuKeycode::Kp6(key_pressed),
            SdlKeycode::Kp7 => LibemuKeycode::Kp7(key_pressed),
            SdlKeycode::Kp8 => LibemuKeycode::Kp8(key_pressed),
            SdlKeycode::Kp9 => LibemuKeycode::Kp9(key_pressed),
            SdlKeycode::Kp0 => LibemuKeycode::Kp0(key_pressed),
            SdlKeycode::KpPeriod => LibemuKeycode::KpPeriod(key_pressed),
            SdlKeycode::Application => LibemuKeycode::Application(key_pressed),
            SdlKeycode::Power => LibemuKeycode::Power(key_pressed),
            SdlKeycode::KpEquals => LibemuKeycode::KpEquals(key_pressed),
            SdlKeycode::F13 => LibemuKeycode::F13(key_pressed),
            SdlKeycode::F14 => LibemuKeycode::F14(key_pressed),
            SdlKeycode::F15 => LibemuKeycode::F15(key_pressed),
            SdlKeycode::F16 => LibemuKeycode::F16(key_pressed),
            SdlKeycode::F17 => LibemuKeycode::F17(key_pressed),
            SdlKeycode::F18 => LibemuKeycode::F18(key_pressed),
            SdlKeycode::F19 => LibemuKeycode::F19(key_pressed),
            SdlKeycode::F20 => LibemuKeycode::F20(key_pressed),
            SdlKeycode::F21 => LibemuKeycode::F21(key_pressed),
            SdlKeycode::F22 => LibemuKeycode::F22(key_pressed),
            SdlKeycode::F23 => LibemuKeycode::F23(key_pressed),
            SdlKeycode::F24 => LibemuKeycode::F24(key_pressed),
            SdlKeycode::Execute => LibemuKeycode::Execute(key_pressed),
            SdlKeycode::Help => LibemuKeycode::Help(key_pressed),
            SdlKeycode::Menu => LibemuKeycode::Menu(key_pressed),
            SdlKeycode::Select => LibemuKeycode::Select(key_pressed),
            SdlKeycode::Stop => LibemuKeycode::Stop(key_pressed),
            SdlKeycode::Again => LibemuKeycode::Again(key_pressed),
            SdlKeycode::Undo => LibemuKeycode::Undo(key_pressed),
            SdlKeycode::Cut => LibemuKeycode::Cut(key_pressed),
            SdlKeycode::Copy => LibemuKeycode::Copy(key_pressed),
            SdlKeycode::Paste => LibemuKeycode::Paste(key_pressed),
            SdlKeycode::Find => LibemuKeycode::Find(key_pressed),
            SdlKeycode::Mute => LibemuKeycode::Mute(key_pressed),
            SdlKeycode::VolumeUp => LibemuKeycode::VolumeUp(key_pressed),
            SdlKeycode::VolumeDown => LibemuKeycode::VolumeDown(key_pressed),
            SdlKeycode::KpComma => LibemuKeycode::KpComma(key_pressed),
            SdlKeycode::KpEqualsAS400 => LibemuKeycode::KpEqualsAS400(key_pressed),
            SdlKeycode::AltErase => LibemuKeycode::AltErase(key_pressed),
            SdlKeycode::Sysreq => LibemuKeycode::Sysreq(key_pressed),
            SdlKeycode::Cancel => LibemuKeycode::Cancel(key_pressed),
            SdlKeycode::Clear => LibemuKeycode::Clear(key_pressed),
            SdlKeycode::Prior => LibemuKeycode::Prior(key_pressed),
            SdlKeycode::Return2 => LibemuKeycode::Return2(key_pressed),
            SdlKeycode::Separator => LibemuKeycode::Separator(key_pressed),
            SdlKeycode::Out => LibemuKeycode::Out(key_pressed),
            SdlKeycode::Oper => LibemuKeycode::Oper(key_pressed),
            SdlKeycode::ClearAgain => LibemuKeycode::ClearAgain(key_pressed),
            SdlKeycode::CrSel => LibemuKeycode::CrSel(key_pressed),
            SdlKeycode::ExSel => LibemuKeycode::ExSel(key_pressed),
            SdlKeycode::Kp00 => LibemuKeycode::Kp00(key_pressed),
            SdlKeycode::Kp000 => LibemuKeycode::Kp000(key_pressed),
            SdlKeycode::ThousandsSeparator => LibemuKeycode::ThousandsSeparator(key_pressed),
            SdlKeycode::DecimalSeparator => LibemuKeycode::DecimalSeparator(key_pressed),
            SdlKeycode::CurrencyUnit => LibemuKeycode::CurrencyUnit(key_pressed),
            SdlKeycode::CurrencySubUnit => LibemuKeycode::CurrencySubUnit(key_pressed),
            SdlKeycode::KpLeftParen => LibemuKeycode::KpLeftParen(key_pressed),
            SdlKeycode::KpRightParen => LibemuKeycode::KpRightParen(key_pressed),
            SdlKeycode::KpLeftBrace => LibemuKeycode::KpLeftBrace(key_pressed),
            SdlKeycode::KpRightBrace => LibemuKeycode::KpRightBrace(key_pressed),
            SdlKeycode::KpTab => LibemuKeycode::KpTab(key_pressed),
            SdlKeycode::KpBackspace => LibemuKeycode::KpBackspace(key_pressed),
            SdlKeycode::KpA => LibemuKeycode::KpA(key_pressed),
            SdlKeycode::KpB => LibemuKeycode::KpB(key_pressed),
            SdlKeycode::KpC => LibemuKeycode::KpC(key_pressed),
            SdlKeycode::KpD => LibemuKeycode::KpD(key_pressed),
            SdlKeycode::KpE => LibemuKeycode::KpE(key_pressed),
            SdlKeycode::KpF => LibemuKeycode::KpF(key_pressed),
            SdlKeycode::KpXor => LibemuKeycode::KpXor(key_pressed),
            SdlKeycode::KpPower => LibemuKeycode::KpPower(key_pressed),
            SdlKeycode::KpPercent => LibemuKeycode::KpPercent(key_pressed),
            SdlKeycode::KpLess => LibemuKeycode::KpLess(key_pressed),
            SdlKeycode::KpGreater => LibemuKeycode::KpGreater(key_pressed),
            SdlKeycode::KpAmpersand => LibemuKeycode::KpAmpersand(key_pressed),
            SdlKeycode::KpDblAmpersand => LibemuKeycode::KpDblAmpersand(key_pressed),
            SdlKeycode::KpVerticalBar => LibemuKeycode::KpVerticalBar(key_pressed),
            SdlKeycode::KpDblVerticalBar => LibemuKeycode::KpDblVerticalBar(key_pressed),
            SdlKeycode::KpColon => LibemuKeycode::KpColon(key_pressed),
            SdlKeycode::KpHash => LibemuKeycode::KpHash(key_pressed),
            SdlKeycode::KpSpace => LibemuKeycode::KpSpace(key_pressed),
            SdlKeycode::KpAt => LibemuKeycode::KpAt(key_pressed),
            SdlKeycode::KpExclam => LibemuKeycode::KpExclam(key_pressed),
            SdlKeycode::KpMemStore => LibemuKeycode::KpMemStore(key_pressed),
            SdlKeycode::KpMemRecall => LibemuKeycode::KpMemRecall(key_pressed),
            SdlKeycode::KpMemClear => LibemuKeycode::KpMemClear(key_pressed),
            SdlKeycode::KpMemAdd => LibemuKeycode::KpMemAdd(key_pressed),
            SdlKeycode::KpMemSubtract => LibemuKeycode::KpMemSubtract(key_pressed),
            SdlKeycode::KpMemMultiply => LibemuKeycode::KpMemMultiply(key_pressed),
            SdlKeycode::KpMemDivide => LibemuKeycode::KpMemDivide(key_pressed),
            SdlKeycode::KpPlusMinus => LibemuKeycode::KpPlusMinus(key_pressed),
            SdlKeycode::KpClear => LibemuKeycode::KpClear(key_pressed),
            SdlKeycode::KpClearEntry => LibemuKeycode::KpClearEntry(key_pressed),
            SdlKeycode::KpBinary => LibemuKeycode::KpBinary(key_pressed),
            SdlKeycode::KpOctal => LibemuKeycode::KpOctal(key_pressed),
            SdlKeycode::KpDecimal => LibemuKeycode::KpDecimal(key_pressed),
            SdlKeycode::KpHexadecimal => LibemuKeycode::KpHexadecimal(key_pressed),
            SdlKeycode::LCtrl => LibemuKeycode::LCtrl(key_pressed),
            SdlKeycode::LShift => LibemuKeycode::LShift(key_pressed),
            SdlKeycode::LAlt => LibemuKeycode::LAlt(key_pressed),
            SdlKeycode::LGui => LibemuKeycode::LGui(key_pressed),
            SdlKeycode::RCtrl => LibemuKeycode::RCtrl(key_pressed),
            SdlKeycode::RShift => LibemuKeycode::RShift(key_pressed),
            SdlKeycode::RAlt => LibemuKeycode::RAlt(key_pressed),
            SdlKeycode::RGui => LibemuKeycode::RGui(key_pressed),
            SdlKeycode::Mode => LibemuKeycode::Mode(key_pressed),
            SdlKeycode::AudioNext => LibemuKeycode::AudioNext(key_pressed),
            SdlKeycode::AudioPrev => LibemuKeycode::AudioPrev(key_pressed),
            SdlKeycode::AudioStop => LibemuKeycode::AudioStop(key_pressed),
            SdlKeycode::AudioPlay => LibemuKeycode::AudioPlay(key_pressed),
            SdlKeycode::AudioMute => LibemuKeycode::AudioMute(key_pressed),
            SdlKeycode::MediaSelect => LibemuKeycode::MediaSelect(key_pressed),
            SdlKeycode::Www => LibemuKeycode::Www(key_pressed),
            SdlKeycode::Mail => LibemuKeycode::Mail(key_pressed),
            SdlKeycode::Calculator => LibemuKeycode::Calculator(key_pressed),
            SdlKeycode::Computer => LibemuKeycode::Computer(key_pressed),
            SdlKeycode::AcSearch => LibemuKeycode::AcSearch(key_pressed),
            SdlKeycode::AcHome => LibemuKeycode::AcHome(key_pressed),
            SdlKeycode::AcBack => LibemuKeycode::AcBack(key_pressed),
            SdlKeycode::AcForward => LibemuKeycode::AcForward(key_pressed),
            SdlKeycode::AcStop => LibemuKeycode::AcStop(key_pressed),
            SdlKeycode::AcRefresh => LibemuKeycode::AcRefresh(key_pressed),
            SdlKeycode::AcBookmarks => LibemuKeycode::AcBookmarks(key_pressed),
            SdlKeycode::BrightnessDown => LibemuKeycode::BrightnessDown(key_pressed),
            SdlKeycode::BrightnessUp => LibemuKeycode::BrightnessUp(key_pressed),
            SdlKeycode::DisplaySwitch => LibemuKeycode::DisplaySwitch(key_pressed),
            SdlKeycode::KbdIllumToggle => LibemuKeycode::KbdIllumToggle(key_pressed),
            SdlKeycode::KbdIllumDown => LibemuKeycode::KbdIllumDown(key_pressed),
            SdlKeycode::KbdIllumUp => LibemuKeycode::KbdIllumUp(key_pressed),
            SdlKeycode::Eject => LibemuKeycode::Eject(key_pressed),
            SdlKeycode::Sleep => LibemuKeycode::Sleep(key_pressed),
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

    fn poll_key_event(&mut self) -> Option<LibemuKeycode> {
        for event in self.event_pump.poll_iter() {
            if let Event::KeyDown { keycode, .. } = event {
                if let Some(keycode) = keycode {
                    return Some(FrontendSdl::sdl_to_io_frontend_keycode(keycode, true));
                }
            } else if let Event::KeyUp { keycode, .. } = event {
                if let Some(keycode) = keycode {
                    return Some(FrontendSdl::sdl_to_io_frontend_keycode(keycode, false));
                }
            }
        }

        None
    }

    fn wait_keypress(&mut self) -> interfaces::Keycode {
        for event in self.event_pump.wait_iter() {
            if let Event::KeyDown { keycode, .. } = event {
                if let Some(keycode) = keycode {
                    return FrontendSdl::sdl_to_io_frontend_keycode(keycode, true);
                }
            }
        }

        unreachable!()
    }
}
