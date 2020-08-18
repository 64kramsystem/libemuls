pub trait IoFrontend {
    fn init(&mut self, screen_width: u32, screen_height: u32);

    fn update_screen(&mut self, pixels: &Vec<(u8, u8, u8)>);

    // The client struct is required to handle also non-input events, e.g. Quit (see EventCode).
    //
    // The boolean field is used for keys, and represented the pressed (true) and released (false)
    // states.
    //
    fn poll_event(&mut self) -> Option<(EventCode, bool)>;
    fn wait_keypress(&mut self) -> EventCode;
}
