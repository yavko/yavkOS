#![no_std]
#![no_main]
#![feature(custom_test_frameworks)]
#![test_runner(yos::test_runner)]
#![reexport_test_harness_main = "test_main"]
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

#[no_mangle]
pub extern "C" fn _start() -> ! {
    println!("{}", LOGO);
    println!("Welcome to yavko's WASM based tiny OS!");

    #[cfg(test)]
    test_main();
    panic!("Nothing else to run");
}

/// This function is called on panic.
#[cfg(not(test))]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    loop {}
}

#[cfg(test)]
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    yos::test_panic_handler(info)
}
