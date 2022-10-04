use virtualization_sys::{self as vz_sys, IVZVirtualMachine};

use crate::{config::VZVirtualMachineConfiguration, foundation::NSError};

pub struct VZVirtualMachine {
    inner: vz_sys::VZVirtualMachine,
    dispatch_queue: vz_sys::NSObject,
}

impl VZVirtualMachine {
    pub fn new(config: VZVirtualMachineConfiguration) -> Self {
        let label = "vm_queue";
        let label_cstr = label.as_ptr() as *const std::os::raw::c_char;
        let null_attrs = vz_sys::NSObject(0 as vz_sys::id);
        let queue = unsafe { vz_sys::dispatch_queue_create(label_cstr, null_attrs) };

        let inner = vz_sys::VZVirtualMachine::alloc();
        let inner = unsafe {
            let ptr = inner.initWithConfiguration_queue_(config.into_inner(), queue);
            vz_sys::VZVirtualMachine(ptr)
        };

        Self {
            inner,
            dispatch_queue: queue,
        }
    }

    pub fn into_inner(self) -> (vz_sys::VZVirtualMachine, vz_sys::NSObject) {
        (self.inner, self.dispatch_queue)
    }

    pub fn start(&self) -> Result<(), NSError> {
        let inner = self.inner;
        let dispatch_block = block::ConcreteBlock::new(move || {
            let completion_handler = block::ConcreteBlock::new(|err: vz_sys::id| {
                println!("in completion handler");

                if err.is_null() {
                    println!("VM started successfully");
                    return;
                }

                let err = NSError::from(err);
                println!("VM failed to start. Error: {:?}", err);
            });
            let completion_handler = completion_handler.copy();
            let completion_handler: &block::Block<(vz_sys::id,), ()> = &completion_handler;
            let completion_handler_ptr: *mut std::os::raw::c_void =
                completion_handler as *const _ as *mut std::os::raw::c_void;
            println!("start_callback_ptr is {:?}", completion_handler_ptr);
            unsafe { inner.startWithCompletionHandler_(completion_handler_ptr) };
            println!("started, awaiting handler");
        });
        let dispatch_block = dispatch_block.copy();
        let dispatch_block: &block::Block<(), ()> = &dispatch_block;
        let dispatch_block_ptr: *mut std::os::raw::c_void =
            dispatch_block as *const _ as *mut std::os::raw::c_void;
        unsafe {
            vz_sys::dispatch_sync(self.dispatch_queue, dispatch_block_ptr);
        }
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
        std::thread::sleep(std::time::Duration::from_secs(30));
        assert!(false);
    }
}
