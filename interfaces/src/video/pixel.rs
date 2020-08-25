use std::ops;

#[derive(Clone, PartialEq)]
pub struct Pixel(pub u8, pub u8, pub u8);

impl ops::BitXorAssign<Pixel> for Pixel {
    fn bitxor_assign(&mut self, rhs: Pixel) {
        self.0 ^= rhs.0;
        self.1 ^= rhs.1;
        self.2 ^= rhs.2;
    }
}

impl Pixel {
    pub const ON: Pixel = Pixel(255, 255, 255);
    pub const OFF: Pixel = Pixel(0, 0, 0);
}
