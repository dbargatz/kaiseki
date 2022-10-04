use virtualization_sys::{self as vz_sys, IVZVirtualMachine};

mod bootloader;
mod config;
pub mod foundation;
mod vm;

pub use bootloader::VZLinuxBootLoader;
pub use config::VZVirtualMachineConfiguration;
pub use vm::VZVirtualMachine;

pub fn supported() -> bool {
    unsafe { vz_sys::VZVirtualMachine::isSupported() }
}

#[cfg(test)]
mod tests {
    use super::supported;

    #[test]
    fn virtualization_supported() {
        assert!(supported())
    }
}
