use virtualization_sys::{IVZVirtualMachine, VZVirtualMachine};

mod bootloader;
mod config;
pub mod foundation;

pub use bootloader::VZLinuxBootLoader;
pub use config::VZVirtualMachineConfiguration;

pub fn supported() -> bool {
    unsafe { VZVirtualMachine::isSupported() }
}

#[cfg(test)]
mod tests {
    use super::supported;

    #[test]
    fn virtualization_supported() {
        assert!(supported())
    }
}
