use virtualization_sys::{
    self as vz_sys, VZVirtualMachineConfiguration_VZVirtualMachineConfigurationValidation,
};
use vz_sys::{INSObject, IVZVirtualMachineConfiguration};

use crate::{bootloader::VZBootLoader, foundation::NSError};

pub struct VZVirtualMachineConfiguration(vz_sys::VZVirtualMachineConfiguration);
impl VZVirtualMachineConfiguration {
    pub fn new() -> Self {
        let inner = vz_sys::VZVirtualMachineConfiguration::alloc();
        let inner = unsafe {
            let ptr = inner.init();
            vz_sys::VZVirtualMachineConfiguration(ptr)
        };
        Self(inner)
    }

    pub fn into_inner(self) -> vz_sys::VZVirtualMachineConfiguration {
        self.0
    }

    pub fn with_bootloader(self, bootloader: VZBootLoader) -> Self {
        unsafe { self.0.setBootLoader_(bootloader.into_inner()) }
        self
    }

    pub fn with_cpus(self, num_cpus: usize) -> Self {
        unsafe { self.0.setCPUCount_(num_cpus as u64) }
        self
    }

    pub fn with_memory(self, num_bytes: usize) -> Self {
        unsafe { self.0.setMemorySize_(num_bytes as u64) }
        self
    }

    pub fn validate_with_error(&self) -> Result<(), NSError> {
        let err = NSError::new();
        let mut err_inner = err.into_inner();
        if unsafe { self.0.validateWithError_(&mut err_inner) } {
            Ok(())
        } else {
            let return_err = NSError::from(err_inner);
            Err(return_err)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::VZVirtualMachineConfiguration;
    use crate::VZLinuxBootLoader;

    #[test]
    fn new_works() {
        let _ = VZVirtualMachineConfiguration::new();
    }

    #[test]
    fn with_bootloader_works() {
        let command_line = "console=hvc0";
        let kernel_path = "/Users/user/Documents/vmlinuz";
        let initrd_path = "/Users/user/Documents/initrd.img";
        let bootloader = VZLinuxBootLoader::new(kernel_path)
            .with_command_line(command_line)
            .with_initial_ramdisk_path(initrd_path);
        let _ = VZVirtualMachineConfiguration::new().with_bootloader(bootloader.into());
    }

    #[test]
    fn with_cpus_works() {
        let _ = VZVirtualMachineConfiguration::new().with_cpus(2);
    }

    #[test]
    fn with_memory_works() {
        let _ = VZVirtualMachineConfiguration::new().with_memory(2 * 1024 * 1024 * 1024);
    }

    #[test]
    fn validate_with_error_works() {
        let command_line = "console=hvc0";
        let kernel_path = "/Users/dylan/Downloads/vmlinux";
        let initrd_path = "/Users/dylan/Downloads/initrd.img";
        let bootloader = VZLinuxBootLoader::new(kernel_path)
            .with_command_line(command_line)
            .with_initial_ramdisk_path(initrd_path);

        let config = VZVirtualMachineConfiguration::new()
            .with_bootloader(bootloader.into())
            .with_cpus(2)
            .with_memory(2 * 1024 * 1024 * 1024);

        config.validate_with_error().expect("VM config was invalid");
    }
}
