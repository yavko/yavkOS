#![no_std]
#![no_main]
#![feature(custom_test_frameworks, try_blocks)]
#![test_runner(kernel::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;
use bootloader_api::config::{BootloaderConfig, Mapping};
use bootloader_api::info::FrameBuffer;
use bootloader_api::{entry_point, info::Optional, BootInfo};
use conquer_once::spin::OnceCell;
use core::panic::PanicInfo;
use kernel::framebuffer::{FrameBufferWriter, FRAMEBUFFER};
use kernel::{println, serial_println};
mod wasm;

static LOGO: &str = r"
                   _     ___  ____  
 _   _  __ ___   _| | __/ _ \/ ___| 
| | | |/ _` \ \ / / |/ / | | \___ \ 
| |_| | (_| |\ V /|   <| |_| |___) |
 \__, |\__,_| \_/ |_|\_\\___/|____/ 
 |___/                              
";

entry_point!(kernel_main, config = &kernel::BOOTLOADER_CONFIG);
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    use kernel::allocator;
    use kernel::memory::{self, BootInfoFrameAllocator};
    use kernel::task::{
        executor::{Executor, Spawner},
        keyboard,
    };
    use x86_64::VirtAddr;
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
    kernel::init();
    let phys_mem_offset = VirtAddr::new(*boot_info.physical_memory_offset.as_ref().unwrap());
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    serial_println!("Still running");
    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap init failed!");
    kernel::mouse::init();
    println!("{}", LOGO);
    println!("Welcome to yavko's WASM based tiny OS!");

    #[cfg(test)]
    test_main();
    let _result: anyhow::Result<()> = try {
        let spawner = Spawner::new(100);
        let mut executor = Executor::new(spawner.clone());
        spawner.add(wasm::example_exec());
        spawner.add(keyboard::print_keypresses());
        spawner.add(kernel::task::mouse::process());
        println!("Still running");
        executor.run();
    };
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    serial_println!("Kernel {}", info);
    println!("Kernel {}", info);
    kernel::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    kernel::test_panic_handler(info)
}
