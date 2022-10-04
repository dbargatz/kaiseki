use virtualization_sys::{self as vz_sys, IVZVirtualMachine};

use crate::{
    config::VZVirtualMachineConfiguration,
    foundation::{DispatchQueue, NSError},
};

pub struct VZVirtualMachine {
    inner: vz_sys::VZVirtualMachine,
    dispatch_queue: DispatchQueue,
}

impl VZVirtualMachine {
    pub fn new(config: VZVirtualMachineConfiguration) -> Self {
        let queue = DispatchQueue::new("vm_queue");
        let inner = vz_sys::VZVirtualMachine::alloc();
        let inner = unsafe {
            let ptr = inner.initWithConfiguration_queue_(config.into_inner(), queue.as_object());
            vz_sys::VZVirtualMachine(ptr)
        };

        Self {
            inner,
            dispatch_queue: queue,
        }
    }

    pub fn into_inner(self) -> (vz_sys::VZVirtualMachine, DispatchQueue) {
        (self.inner, self.dispatch_queue)
    }

    pub fn start(&self) -> Result<(), NSError> {
        let inner = self.inner;
        let dispatch_closure = move || {
            let completion_handler = block::ConcreteBlock::new(|err: vz_sys::id| {
                println!("in completion handler");

                if err.is_null() {
                    println!("VM started successfully");
                    return;
                }

                let err = NSError::from(err);
                println!("VM failed to start. {}", err);
            });
            let completion_handler = completion_handler.copy();
            let completion_handler: &block::Block<(vz_sys::id,), ()> = &completion_handler;
            let completion_handler_ptr: *mut std::os::raw::c_void =
                completion_handler as *const _ as *mut std::os::raw::c_void;
            unsafe { inner.startWithCompletionHandler_(completion_handler_ptr) };
        };
        self.dispatch_queue.dispatch_async(dispatch_closure);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::VZVirtualMachine;
    use crate::{VZLinuxBootLoader, VZVirtualMachineConfiguration};

    fn create_linux_config() -> VZVirtualMachineConfiguration {
        let command_line = "console=hvc0";
        let kernel_path = "/Users/user/Downloads/vmlinuz";
        let initrd_path = "/Users/user/Downloads/initrd.img";
        let bootloader = VZLinuxBootLoader::new(kernel_path)
            .with_command_line(command_line)
            .with_initial_ramdisk_path(initrd_path);

        let config = VZVirtualMachineConfiguration::new()
            .with_bootloader(bootloader.into())
            .with_cpus(2)
            .with_memory(2 * 1024 * 1024 * 1024);

        config
    }

    #[test]
    fn new_works() {
        let config = create_linux_config();
        let _ = VZVirtualMachine::new(config);
    }

    #[test]
    fn start_works() {
        let config = create_linux_config();
        let vm = VZVirtualMachine::new(config);
        vm.start().expect("VM failed to start");
    }
}
