#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(yos::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use yos::{print, println};

static LOGO: &str = r"
                   _     ___  ____  
 _   _  __ ___   _| | __/ _ \/ ___| 
| | | |/ _` \ \ / / |/ / | | \___ \ 
| |_| | (_| |\ V /|   <| |_| |___) |
 \__, |\__,_| \_/ |_|\_\\___/|____/ 
 |___/                              
";

entry_point!(kernel_main);
fn kernel_main(boot_info: &'static BootInfo) -> ! {
    use x86_64::VirtAddr;
    use yos::allocator;
    use yos::memory::{self, BootInfoFrameAllocator};
    println!("{}", LOGO);
    println!("Welcome to yavko's WASM based tiny OS!");

    yos::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mut mapper = unsafe { memory::init(phys_mem_offset) };
    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_map) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap init failed!");

    #[cfg(test)]
    test_main();
    yos::hlt_loop();
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("Kernel {}", info);
    yos::hlt_loop();
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    yos::test_panic_handler(info)
}
