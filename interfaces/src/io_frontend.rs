pub trait IoFrontend {
    fn init(&mut self, screen_width: u32, screen_height: u32);

    fn draw_pixel(&mut self, x: u32, y: u32, r: u8, g: u8, b: u8);
    fn update_screen(&mut self);

    // true/false for key pressed/released.
    //
    fn poll_key_event(&mut self) -> Option<(Keycode, bool)>;
    fn wait_keypress(&mut self) -> Keycode;
}
