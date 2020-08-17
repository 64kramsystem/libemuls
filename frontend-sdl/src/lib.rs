use interfaces::{EventCode, IoFrontend};

use sdl2::{event::Event, pixels::Color, rect::Point, render::Canvas, video::Window};
use std::collections::HashMap;

use sdl2::{keyboard::Keycode as SdlKeycode, EventPump};

pub struct FrontendSdl {
    event_pump: EventPump,
    canvas: Canvas<Window>,
    custom_keys_mapping: HashMap<EventCode, EventCode>,
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

        FrontendSdl {
            event_pump,
            canvas,
            custom_keys_mapping,
        }
    }

    // Ugly but necessary, as we can't trivially map an enum to another enum
    //
    fn sdl_to_io_frontend_keycode(sdl_keycode: SdlKeycode, key_pressed: bool) -> EventCode {
        match sdl_keycode {
            SdlKeycode::Backspace => EventCode::KeyBackspace(key_pressed),
            SdlKeycode::Tab => EventCode::KeyTab(key_pressed),
            SdlKeycode::Return => EventCode::KeyReturn(key_pressed),
            SdlKeycode::Escape => EventCode::KeyEscape(key_pressed),
            SdlKeycode::Space => EventCode::KeySpace(key_pressed),
            SdlKeycode::Exclaim => EventCode::KeyExclaim(key_pressed),
            SdlKeycode::Quotedbl => EventCode::KeyQuotedbl(key_pressed),
            SdlKeycode::Hash => EventCode::KeyHash(key_pressed),
            SdlKeycode::Dollar => EventCode::KeyDollar(key_pressed),
            SdlKeycode::Percent => EventCode::KeyPercent(key_pressed),
            SdlKeycode::Ampersand => EventCode::KeyAmpersand(key_pressed),
            SdlKeycode::Quote => EventCode::KeyQuote(key_pressed),
            SdlKeycode::LeftParen => EventCode::KeyLeftParen(key_pressed),
            SdlKeycode::RightParen => EventCode::KeyRightParen(key_pressed),
            SdlKeycode::Asterisk => EventCode::KeyAsterisk(key_pressed),
            SdlKeycode::Plus => EventCode::KeyPlus(key_pressed),
            SdlKeycode::Comma => EventCode::KeyComma(key_pressed),
            SdlKeycode::Minus => EventCode::KeyMinus(key_pressed),
            SdlKeycode::Period => EventCode::KeyPeriod(key_pressed),
            SdlKeycode::Slash => EventCode::KeySlash(key_pressed),
            SdlKeycode::Num0 => EventCode::KeyNum0(key_pressed),
            SdlKeycode::Num1 => EventCode::KeyNum1(key_pressed),
            SdlKeycode::Num2 => EventCode::KeyNum2(key_pressed),
            SdlKeycode::Num3 => EventCode::KeyNum3(key_pressed),
            SdlKeycode::Num4 => EventCode::KeyNum4(key_pressed),
            SdlKeycode::Num5 => EventCode::KeyNum5(key_pressed),
            SdlKeycode::Num6 => EventCode::KeyNum6(key_pressed),
            SdlKeycode::Num7 => EventCode::KeyNum7(key_pressed),
            SdlKeycode::Num8 => EventCode::KeyNum8(key_pressed),
            SdlKeycode::Num9 => EventCode::KeyNum9(key_pressed),
            SdlKeycode::Colon => EventCode::KeyColon(key_pressed),
            SdlKeycode::Semicolon => EventCode::KeySemicolon(key_pressed),
            SdlKeycode::Less => EventCode::KeyLess(key_pressed),
            SdlKeycode::Equals => EventCode::KeyEquals(key_pressed),
            SdlKeycode::Greater => EventCode::KeyGreater(key_pressed),
            SdlKeycode::Question => EventCode::KeyQuestion(key_pressed),
            SdlKeycode::At => EventCode::KeyAt(key_pressed),
            SdlKeycode::LeftBracket => EventCode::KeyLeftBracket(key_pressed),
            SdlKeycode::Backslash => EventCode::KeyBackslash(key_pressed),
            SdlKeycode::RightBracket => EventCode::KeyRightBracket(key_pressed),
            SdlKeycode::Caret => EventCode::KeyCaret(key_pressed),
            SdlKeycode::Underscore => EventCode::KeyUnderscore(key_pressed),
            SdlKeycode::Backquote => EventCode::KeyBackquote(key_pressed),
            SdlKeycode::A => EventCode::KeyA(key_pressed),
            SdlKeycode::B => EventCode::KeyB(key_pressed),
            SdlKeycode::C => EventCode::KeyC(key_pressed),
            SdlKeycode::D => EventCode::KeyD(key_pressed),
            SdlKeycode::E => EventCode::KeyE(key_pressed),
            SdlKeycode::F => EventCode::KeyF(key_pressed),
            SdlKeycode::G => EventCode::KeyG(key_pressed),
            SdlKeycode::H => EventCode::KeyH(key_pressed),
            SdlKeycode::I => EventCode::KeyI(key_pressed),
            SdlKeycode::J => EventCode::KeyJ(key_pressed),
            SdlKeycode::K => EventCode::KeyK(key_pressed),
            SdlKeycode::L => EventCode::KeyL(key_pressed),
            SdlKeycode::M => EventCode::KeyM(key_pressed),
            SdlKeycode::N => EventCode::KeyN(key_pressed),
            SdlKeycode::O => EventCode::KeyO(key_pressed),
            SdlKeycode::P => EventCode::KeyP(key_pressed),
            SdlKeycode::Q => EventCode::KeyQ(key_pressed),
            SdlKeycode::R => EventCode::KeyR(key_pressed),
            SdlKeycode::S => EventCode::KeyS(key_pressed),
            SdlKeycode::T => EventCode::KeyT(key_pressed),
            SdlKeycode::U => EventCode::KeyU(key_pressed),
            SdlKeycode::V => EventCode::KeyV(key_pressed),
            SdlKeycode::W => EventCode::KeyW(key_pressed),
            SdlKeycode::X => EventCode::KeyX(key_pressed),
            SdlKeycode::Y => EventCode::KeyY(key_pressed),
            SdlKeycode::Z => EventCode::KeyZ(key_pressed),
            SdlKeycode::Delete => EventCode::KeyDelete(key_pressed),
            SdlKeycode::CapsLock => EventCode::KeyCapsLock(key_pressed),
            SdlKeycode::F1 => EventCode::KeyF1(key_pressed),
            SdlKeycode::F2 => EventCode::KeyF2(key_pressed),
            SdlKeycode::F3 => EventCode::KeyF3(key_pressed),
            SdlKeycode::F4 => EventCode::KeyF4(key_pressed),
            SdlKeycode::F5 => EventCode::KeyF5(key_pressed),
            SdlKeycode::F6 => EventCode::KeyF6(key_pressed),
            SdlKeycode::F7 => EventCode::KeyF7(key_pressed),
            SdlKeycode::F8 => EventCode::KeyF8(key_pressed),
            SdlKeycode::F9 => EventCode::KeyF9(key_pressed),
            SdlKeycode::F10 => EventCode::KeyF10(key_pressed),
            SdlKeycode::F11 => EventCode::KeyF11(key_pressed),
            SdlKeycode::F12 => EventCode::KeyF12(key_pressed),
            SdlKeycode::PrintScreen => EventCode::KeyPrintScreen(key_pressed),
            SdlKeycode::ScrollLock => EventCode::KeyScrollLock(key_pressed),
            SdlKeycode::Pause => EventCode::KeyPause(key_pressed),
            SdlKeycode::Insert => EventCode::KeyInsert(key_pressed),
            SdlKeycode::Home => EventCode::KeyHome(key_pressed),
            SdlKeycode::PageUp => EventCode::KeyPageUp(key_pressed),
            SdlKeycode::End => EventCode::KeyEnd(key_pressed),
            SdlKeycode::PageDown => EventCode::KeyPageDown(key_pressed),
            SdlKeycode::Right => EventCode::KeyRight(key_pressed),
            SdlKeycode::Left => EventCode::KeyLeft(key_pressed),
            SdlKeycode::Down => EventCode::KeyDown(key_pressed),
            SdlKeycode::Up => EventCode::KeyUp(key_pressed),
            SdlKeycode::NumLockClear => EventCode::KeyNumLockClear(key_pressed),
            SdlKeycode::KpDivide => EventCode::KeyKpDivide(key_pressed),
            SdlKeycode::KpMultiply => EventCode::KeyKpMultiply(key_pressed),
            SdlKeycode::KpMinus => EventCode::KeyKpMinus(key_pressed),
            SdlKeycode::KpPlus => EventCode::KeyKpPlus(key_pressed),
            SdlKeycode::KpEnter => EventCode::KeyKpEnter(key_pressed),
            SdlKeycode::Kp1 => EventCode::KeyKp1(key_pressed),
            SdlKeycode::Kp2 => EventCode::KeyKp2(key_pressed),
            SdlKeycode::Kp3 => EventCode::KeyKp3(key_pressed),
            SdlKeycode::Kp4 => EventCode::KeyKp4(key_pressed),
            SdlKeycode::Kp5 => EventCode::KeyKp5(key_pressed),
            SdlKeycode::Kp6 => EventCode::KeyKp6(key_pressed),
            SdlKeycode::Kp7 => EventCode::KeyKp7(key_pressed),
            SdlKeycode::Kp8 => EventCode::KeyKp8(key_pressed),
            SdlKeycode::Kp9 => EventCode::KeyKp9(key_pressed),
            SdlKeycode::Kp0 => EventCode::KeyKp0(key_pressed),
            SdlKeycode::KpPeriod => EventCode::KeyKpPeriod(key_pressed),
            SdlKeycode::Application => EventCode::KeyApplication(key_pressed),
            SdlKeycode::Power => EventCode::KeyPower(key_pressed),
            SdlKeycode::KpEquals => EventCode::KeyKpEquals(key_pressed),
            SdlKeycode::F13 => EventCode::KeyF13(key_pressed),
            SdlKeycode::F14 => EventCode::KeyF14(key_pressed),
            SdlKeycode::F15 => EventCode::KeyF15(key_pressed),
            SdlKeycode::F16 => EventCode::KeyF16(key_pressed),
            SdlKeycode::F17 => EventCode::KeyF17(key_pressed),
            SdlKeycode::F18 => EventCode::KeyF18(key_pressed),
            SdlKeycode::F19 => EventCode::KeyF19(key_pressed),
            SdlKeycode::F20 => EventCode::KeyF20(key_pressed),
            SdlKeycode::F21 => EventCode::KeyF21(key_pressed),
            SdlKeycode::F22 => EventCode::KeyF22(key_pressed),
            SdlKeycode::F23 => EventCode::KeyF23(key_pressed),
            SdlKeycode::F24 => EventCode::KeyF24(key_pressed),
            SdlKeycode::Execute => EventCode::KeyExecute(key_pressed),
            SdlKeycode::Help => EventCode::KeyHelp(key_pressed),
            SdlKeycode::Menu => EventCode::KeyMenu(key_pressed),
            SdlKeycode::Select => EventCode::KeySelect(key_pressed),
            SdlKeycode::Stop => EventCode::KeyStop(key_pressed),
            SdlKeycode::Again => EventCode::KeyAgain(key_pressed),
            SdlKeycode::Undo => EventCode::KeyUndo(key_pressed),
            SdlKeycode::Cut => EventCode::KeyCut(key_pressed),
            SdlKeycode::Copy => EventCode::KeyCopy(key_pressed),
            SdlKeycode::Paste => EventCode::KeyPaste(key_pressed),
            SdlKeycode::Find => EventCode::KeyFind(key_pressed),
            SdlKeycode::Mute => EventCode::KeyMute(key_pressed),
            SdlKeycode::VolumeUp => EventCode::KeyVolumeUp(key_pressed),
            SdlKeycode::VolumeDown => EventCode::KeyVolumeDown(key_pressed),
            SdlKeycode::KpComma => EventCode::KeyKpComma(key_pressed),
            SdlKeycode::KpEqualsAS400 => EventCode::KeyKpEqualsAS400(key_pressed),
            SdlKeycode::AltErase => EventCode::KeyAltErase(key_pressed),
            SdlKeycode::Sysreq => EventCode::KeySysreq(key_pressed),
            SdlKeycode::Cancel => EventCode::KeyCancel(key_pressed),
            SdlKeycode::Clear => EventCode::KeyClear(key_pressed),
            SdlKeycode::Prior => EventCode::KeyPrior(key_pressed),
            SdlKeycode::Return2 => EventCode::KeyReturn2(key_pressed),
            SdlKeycode::Separator => EventCode::KeySeparator(key_pressed),
            SdlKeycode::Out => EventCode::KeyOut(key_pressed),
            SdlKeycode::Oper => EventCode::KeyOper(key_pressed),
            SdlKeycode::ClearAgain => EventCode::KeyClearAgain(key_pressed),
            SdlKeycode::CrSel => EventCode::KeyCrSel(key_pressed),
            SdlKeycode::ExSel => EventCode::KeyExSel(key_pressed),
            SdlKeycode::Kp00 => EventCode::KeyKp00(key_pressed),
            SdlKeycode::Kp000 => EventCode::KeyKp000(key_pressed),
            SdlKeycode::ThousandsSeparator => EventCode::KeyThousandsSeparator(key_pressed),
            SdlKeycode::DecimalSeparator => EventCode::KeyDecimalSeparator(key_pressed),
            SdlKeycode::CurrencyUnit => EventCode::KeyCurrencyUnit(key_pressed),
            SdlKeycode::CurrencySubUnit => EventCode::KeyCurrencySubUnit(key_pressed),
            SdlKeycode::KpLeftParen => EventCode::KeyKpLeftParen(key_pressed),
            SdlKeycode::KpRightParen => EventCode::KeyKpRightParen(key_pressed),
            SdlKeycode::KpLeftBrace => EventCode::KeyKpLeftBrace(key_pressed),
            SdlKeycode::KpRightBrace => EventCode::KeyKpRightBrace(key_pressed),
            SdlKeycode::KpTab => EventCode::KeyKpTab(key_pressed),
            SdlKeycode::KpBackspace => EventCode::KeyKpBackspace(key_pressed),
            SdlKeycode::KpA => EventCode::KeyKpA(key_pressed),
            SdlKeycode::KpB => EventCode::KeyKpB(key_pressed),
            SdlKeycode::KpC => EventCode::KeyKpC(key_pressed),
            SdlKeycode::KpD => EventCode::KeyKpD(key_pressed),
            SdlKeycode::KpE => EventCode::KeyKpE(key_pressed),
            SdlKeycode::KpF => EventCode::KeyKpF(key_pressed),
            SdlKeycode::KpXor => EventCode::KeyKpXor(key_pressed),
            SdlKeycode::KpPower => EventCode::KeyKpPower(key_pressed),
            SdlKeycode::KpPercent => EventCode::KeyKpPercent(key_pressed),
            SdlKeycode::KpLess => EventCode::KeyKpLess(key_pressed),
            SdlKeycode::KpGreater => EventCode::KeyKpGreater(key_pressed),
            SdlKeycode::KpAmpersand => EventCode::KeyKpAmpersand(key_pressed),
            SdlKeycode::KpDblAmpersand => EventCode::KeyKpDblAmpersand(key_pressed),
            SdlKeycode::KpVerticalBar => EventCode::KeyKpVerticalBar(key_pressed),
            SdlKeycode::KpDblVerticalBar => EventCode::KeyKpDblVerticalBar(key_pressed),
            SdlKeycode::KpColon => EventCode::KeyKpColon(key_pressed),
            SdlKeycode::KpHash => EventCode::KeyKpHash(key_pressed),
            SdlKeycode::KpSpace => EventCode::KeyKpSpace(key_pressed),
            SdlKeycode::KpAt => EventCode::KeyKpAt(key_pressed),
            SdlKeycode::KpExclam => EventCode::KeyKpExclam(key_pressed),
            SdlKeycode::KpMemStore => EventCode::KeyKpMemStore(key_pressed),
            SdlKeycode::KpMemRecall => EventCode::KeyKpMemRecall(key_pressed),
            SdlKeycode::KpMemClear => EventCode::KeyKpMemClear(key_pressed),
            SdlKeycode::KpMemAdd => EventCode::KeyKpMemAdd(key_pressed),
            SdlKeycode::KpMemSubtract => EventCode::KeyKpMemSubtract(key_pressed),
            SdlKeycode::KpMemMultiply => EventCode::KeyKpMemMultiply(key_pressed),
            SdlKeycode::KpMemDivide => EventCode::KeyKpMemDivide(key_pressed),
            SdlKeycode::KpPlusMinus => EventCode::KeyKpPlusMinus(key_pressed),
            SdlKeycode::KpClear => EventCode::KeyKpClear(key_pressed),
            SdlKeycode::KpClearEntry => EventCode::KeyKpClearEntry(key_pressed),
            SdlKeycode::KpBinary => EventCode::KeyKpBinary(key_pressed),
            SdlKeycode::KpOctal => EventCode::KeyKpOctal(key_pressed),
            SdlKeycode::KpDecimal => EventCode::KeyKpDecimal(key_pressed),
            SdlKeycode::KpHexadecimal => EventCode::KeyKpHexadecimal(key_pressed),
            SdlKeycode::LCtrl => EventCode::KeyLCtrl(key_pressed),
            SdlKeycode::LShift => EventCode::KeyLShift(key_pressed),
            SdlKeycode::LAlt => EventCode::KeyLAlt(key_pressed),
            SdlKeycode::LGui => EventCode::KeyLGui(key_pressed),
            SdlKeycode::RCtrl => EventCode::KeyRCtrl(key_pressed),
            SdlKeycode::RShift => EventCode::KeyRShift(key_pressed),
            SdlKeycode::RAlt => EventCode::KeyRAlt(key_pressed),
            SdlKeycode::RGui => EventCode::KeyRGui(key_pressed),
            SdlKeycode::Mode => EventCode::KeyMode(key_pressed),
            SdlKeycode::AudioNext => EventCode::KeyAudioNext(key_pressed),
            SdlKeycode::AudioPrev => EventCode::KeyAudioPrev(key_pressed),
            SdlKeycode::AudioStop => EventCode::KeyAudioStop(key_pressed),
            SdlKeycode::AudioPlay => EventCode::KeyAudioPlay(key_pressed),
            SdlKeycode::AudioMute => EventCode::KeyAudioMute(key_pressed),
            SdlKeycode::MediaSelect => EventCode::KeyMediaSelect(key_pressed),
            SdlKeycode::Www => EventCode::KeyWww(key_pressed),
            SdlKeycode::Mail => EventCode::KeyMail(key_pressed),
            SdlKeycode::Calculator => EventCode::KeyCalculator(key_pressed),
            SdlKeycode::Computer => EventCode::KeyComputer(key_pressed),
            SdlKeycode::AcSearch => EventCode::KeyAcSearch(key_pressed),
            SdlKeycode::AcHome => EventCode::KeyAcHome(key_pressed),
            SdlKeycode::AcBack => EventCode::KeyAcBack(key_pressed),
            SdlKeycode::AcForward => EventCode::KeyAcForward(key_pressed),
            SdlKeycode::AcStop => EventCode::KeyAcStop(key_pressed),
            SdlKeycode::AcRefresh => EventCode::KeyAcRefresh(key_pressed),
            SdlKeycode::AcBookmarks => EventCode::KeyAcBookmarks(key_pressed),
            SdlKeycode::BrightnessDown => EventCode::KeyBrightnessDown(key_pressed),
            SdlKeycode::BrightnessUp => EventCode::KeyBrightnessUp(key_pressed),
            SdlKeycode::DisplaySwitch => EventCode::KeyDisplaySwitch(key_pressed),
            SdlKeycode::KbdIllumToggle => EventCode::KeyKbdIllumToggle(key_pressed),
            SdlKeycode::KbdIllumDown => EventCode::KeyKbdIllumDown(key_pressed),
            SdlKeycode::KbdIllumUp => EventCode::KeyKbdIllumUp(key_pressed),
            SdlKeycode::Eject => EventCode::KeyEject(key_pressed),
            SdlKeycode::Sleep => EventCode::KeySleep(key_pressed),
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

    fn poll_event(&mut self) -> Option<EventCode> {
        let mut event_code = None;

        for event in self.event_pump.poll_iter() {
            if let Event::KeyDown { keycode, .. } = event {
                if let Some(keycode) = keycode {
                    event_code = Some(FrontendSdl::sdl_to_io_frontend_keycode(keycode, true));
                    break;
                }
            } else if let Event::KeyUp { keycode, .. } = event {
                if let Some(keycode) = keycode {
                    event_code = Some(FrontendSdl::sdl_to_io_frontend_keycode(keycode, false));
                    break;
                }
            } else if let Event::Quit { .. } = event {
                event_code = Some(EventCode::Quit);
                break;
            }
        }

        if let Some(event_code) = event_code {
            if let Some(mapped_event_code) = self.custom_keys_mapping.get(&event_code) {
                Some(mapped_event_code.clone())
            } else {
                Some(event_code)
            }
        } else {
            None
        }
    }

    fn wait_keypress(&mut self) -> interfaces::EventCode {
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
