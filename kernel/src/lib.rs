#![no_std]
#![cfg_attr(test, no_main)]
#![feature(custom_test_frameworks, abi_x86_interrupt, alloc_error_handler)]
#![test_runner(crate::test_runner)]
#![reexport_test_harness_main = "test_main"]

use bootloader_api::{
    config::{BootloaderConfig, Mapping},
    info::FrameBufferInfo,
};
use bootloader_api::{entry_point, BootInfo};
use core::panic::PanicInfo;
extern crate alloc;
pub mod allocator;
pub mod framebuffer;
pub mod gdt;
pub mod interrupts;
pub mod memory;
pub mod mouse;
pub mod serial;
pub mod task;
//pub mod vga_buffer;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u32)]
pub enum QemuExitCode {
    Success = 0x10,
    Failed = 0x11,
}

pub fn exit_qemu(exit_code: QemuExitCode) {
    use x86_64::instructions::port::Port;

    unsafe {
        let mut port = Port::new(0xf4);
        port.write(exit_code as u32);
    }
}

pub trait Testable {
    fn run(&self);
}

impl<T> Testable for T
where
    T: Fn(),
{
    fn run(&self) {
        print!("{}...\t", core::any::type_name::<T>());
        self();
        println!("[ok]");
    }
}

pub fn test_runner(tests: &[&dyn Testable]) {
    println!("Running {} tests", tests.len());
    for test in tests {
        test.run();
    }
    exit_qemu(QemuExitCode::Success);
}

#[allow(clippy::empty_loop)]
pub fn test_panic_handler(info: &PanicInfo) -> ! {
    print!("[failed]\n");
    print!("Error: {}\n", info);
    exit_qemu(QemuExitCode::Failed);
    hlt_loop();
}

pub fn init() {
    gdt::init();
    interrupts::init_idt();
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[alloc_error_handler]
fn alloc_error_handler(layout: alloc::alloc::Layout) -> ! {
    panic!("allocation error: {layout:?}")
}

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

#[cfg(test)]
entry_point!(test_kernel_main, config = &BOOTLOADER_CONFIG);

/// Entry point for `cargo test`
#[cfg(test)]
fn test_kernel_main(boot_info: &'static mut BootInfo) -> ! {
    use crate::framebuffer::{FrameBufferWriter, FRAMEBUFFER};
    init();
    FRAMEBUFFER.init_once(|| {
        let frame = boot_info.framebuffer.as_mut();
        let info = match frame {
            Some(ref v) => v.info(),
            None => panic!("BOOTLOADER NOT CONFIGURED TO SUPPORT FRAMEBUFFER"),
        };
        let buffer = match frame {
            Some(v) => v.buffer_mut(),
            None => panic!("BOOTLOADER NOT CONFIGURED TO SUPPORT FRAMEBUFFER"),
        };
        spinning_top::Spinlock::new(FrameBufferWriter::new(buffer, info))
    });

    test_main();
    hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    test_panic_handler(info)
}
