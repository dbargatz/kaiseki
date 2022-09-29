use virtualization_sys as vz_sys;
use vz_sys::IVZLinuxBootLoader;

use crate::foundation::{NSString, NSURL};

pub struct VZBootLoader(vz_sys::VZBootLoader);
impl VZBootLoader {
    pub fn into_inner(self) -> vz_sys::VZBootLoader {
        self.0
    }
}

impl From<vz_sys::VZLinuxBootLoader> for VZBootLoader {
    fn from(p: vz_sys::VZLinuxBootLoader) -> Self {
        Self(p.into())
    }
}

impl From<VZLinuxBootLoader> for VZBootLoader {
    fn from(p: VZLinuxBootLoader) -> Self {
        Self::from(p.into_inner())
    }
}

pub struct VZLinuxBootLoader(vz_sys::VZLinuxBootLoader);
impl VZLinuxBootLoader {
    pub fn new() -> Self {
        let inner = vz_sys::VZLinuxBootLoader::alloc();
        Self(inner)
    }

    pub fn into_inner(self) -> vz_sys::VZLinuxBootLoader {
        self.0
    }

    pub fn with_command_line(self, command_line: &str) -> Self {
        let cmd_line = NSString::new(command_line);
        unsafe {
            self.0.setCommandLine_(cmd_line.into_inner());
        };
        self
    }

    pub fn with_initial_ramdisk_path(self, path: &str) -> Self {
        let path = NSURL::new(path);
        unsafe {
            self.0.setInitialRamdiskURL_(path.into_inner());
        };
        self
    }

    pub fn with_kernel_path(self, path: &str) -> Self {
        let path = NSURL::new(path);
        unsafe {
            self.0.initWithKernelURL_(path.into_inner());
        };
        self
    }
}

impl From<vz_sys::VZLinuxBootLoader> for VZLinuxBootLoader {
    fn from(p: vz_sys::VZLinuxBootLoader) -> Self {
        Self(p)
    }
}

#[cfg(test)]
mod tests {
    use super::{VZBootLoader, VZLinuxBootLoader};

    #[test]
    fn new_works() {
        let _ = VZLinuxBootLoader::new();
    }

    #[test]
    fn with_command_line_works() {
        let command_line = "console=hvc0";
        let _ = VZLinuxBootLoader::new().with_command_line(command_line);
    }

    #[test]
    fn with_kernel_path_works() {
        let path = "/Users/user/Documents/vmlinuz";
        let _ = VZLinuxBootLoader::new().with_kernel_path(path);
    }

    #[test]
    fn with_initial_ramdisk_path_works() {
        let path = "/Users/user/Documents/initrd.img";
        let _ = VZLinuxBootLoader::new().with_initial_ramdisk_path(path);
    }

    #[test]
    fn into_vzbootloader_works() {
        let command_line = "console=hvc0";
        let kernel_path = "/Users/user/Documents/vmlinuz";
        let initrd_path = "/Users/user/Documents/initrd.img";
        let bootloader = VZLinuxBootLoader::new()
            .with_command_line(command_line)
            .with_initial_ramdisk_path(initrd_path)
            .with_kernel_path(kernel_path);
        let _: VZBootLoader = bootloader.into();
    }
}
