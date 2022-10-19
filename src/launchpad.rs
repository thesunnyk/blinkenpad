
pub trait PadHandler {
    fn on_pad(&self, x: u8, y: u8);
}

pub trait PadArea {
    fn set_light(&self, x: u8, y: u8);
    fn set_handler(&self, handler: dyn PadHandler);
}
