use crate::event_code::EventCode;

pub trait IoFrontend {
    fn init(&mut self, screen_width: u32, screen_height: u32);

    fn update_screen(&mut self, pixels: &[(u8, u8, u8)]);

    fn beep(&mut self);

    // The client code is required to handle also non-input events, e.g. Quit (see EventCode).
    //
    fn poll_event(&mut self) -> Option<EventCode>;
    fn wait_keypress(&mut self) -> EventCode;
}
