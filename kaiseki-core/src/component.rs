use crate::bus::BusConnection;

pub trait Component {
    fn connect_to_bus(&mut self, bus: BusConnection);
    fn start(&mut self);
}
