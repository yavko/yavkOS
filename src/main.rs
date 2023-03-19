#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(yos::test_runner)]
#![reexport_test_harness_main = "test_main"]
use bootloader::{entry_point, BootInfo};
use core::panic::PanicInfo;
use yos::println;

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
    use x86_64::{structures::paging::Translate, VirtAddr};
    use yos::memory;
    println!("{}", LOGO);
    println!("Welcome to yavko's WASM based tiny OS!");

    yos::init();

    let phys_mem_offset = VirtAddr::new(boot_info.physical_memory_offset);
    let mapper = unsafe { memory::init(phys_mem_offset) };

    let addresses = [
        // the identity-mapped vga buffer page
        0xb8000,
        // some code page
        0x201008,
        // some stack page
        0x0100_0020_1a10,
        // virtual address mapped to physical address 0
        boot_info.physical_memory_offset,
    ];

    for &address in &addresses {
        let virt = VirtAddr::new(address);
        let phys = mapper.translate_addr(virt);
        println!("{:?} -> {:?}", virt, phys);
    }

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
