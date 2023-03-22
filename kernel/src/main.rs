#![no_std]
#![no_main]
#![feature(custom_test_frameworks, try_blocks)]
#![test_runner(yos::test_runner)]
#![reexport_test_harness_main = "test_main"]
extern crate alloc;
use bootloader_api::config::{BootloaderConfig, Mapping};
use bootloader_api::{entry_point, info::Optional, BootInfo};
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

pub static BOOTLOADER_CONFIG: BootloaderConfig = {
    let mut config = BootloaderConfig::new_default();
    config.mappings.physical_memory = Some(Mapping::Dynamic);
    config
};

entry_point!(kernel_main, config = &BOOTLOADER_CONFIG);
fn kernel_main(boot_info: &'static mut BootInfo) -> ! {
    use x86_64::VirtAddr;
    use yos::allocator;
    use yos::memory::{self, BootInfoFrameAllocator};
    use yos::task::{
        executor::{Executor, Spawner},
        keyboard, Task,
    };
    println!("{}", LOGO);
    println!("Welcome to yavko's WASM based tiny OS!");

    yos::init();

    let phys_mem_offset = VirtAddr::new(match boot_info.physical_memory_offset {
        Optional::Some(value) => value,
        Optional::None => panic!("BOOTLOADER NOT CONFIGURED TO MAP PHYSICAL MEMORY"),
    });
    let mut mapper = unsafe { memory::init(phys_mem_offset) };

    let mut frame_allocator = unsafe { BootInfoFrameAllocator::init(&boot_info.memory_regions) };

    allocator::init_heap(&mut mapper, &mut frame_allocator).expect("Heap init failed!");

    #[cfg(test)]
    test_main();

    let result: anyhow::Result<()> = try {
        let spawner = Spawner::new(100);
        let mut executor = Executor::new(spawner.clone());
        executor.spawn(Task::new(keyboard::print_keypresses()));
        //executor.spawn(Task::new(example_exec()));
        executor.run();
    };
    result.unwrap();
}
/*
async fn example_exec() {
    use wasmi::*;
    // First step is to create the Wasm execution engine with some config.
    // In this example we are using the default configuration.
    let engine = Engine::default();
    let wat = r#"
        (module
            (import "host" "hello" (func $host_hello (param i32)))
            (func (export "hello")
                (call $host_hello (i32.const 3))
            )
        )
    "#;
    // Wasmi does not yet support parsing `.wat` so we have to convert
    // out `.wat` into `.wasm` before we compile and validate it.
    //let wasm = wat::parse_str(wat).unwrap();
    let wasm = include_bytes!("test.wasm");
    let module = Module::new(&engine, &mut &wasm[..]).unwrap();

    // All Wasm objects operate within the context of a `Store`.
    // Each `Store` has a type parameter to store host-specific data,
    // which in this case we are using `42` for.
    type HostState = u32;
    let mut store = Store::new(&engine, 42);
    let host_hello = Func::wrap(&mut store, |caller: Caller<'_, HostState>, param: i32| {
        println!("Got {param} from WebAssembly");
        println!("My host state is: {}", caller.data());
    });

    // In order to create Wasm module instances and link their imports
    // and exports we require a `Linker`.
    let mut linker = <Linker<HostState>>::new(&engine);
    // Instantiation of a Wasm module requires defining its imports and then
    // afterwards we can fetch exports by name, as well as asserting the
    // type signature of the function with `get_typed_func`.
    //
    // Also before using an instance created this way we need to start it.
    linker.define("host", "hello", host_hello).unwrap();
    let instance = linker.instantiate(&mut store, &module).unwrap().start(&mut store).unwrap();
    let hello = instance.get_typed_func::<(), ()>(&store, "hello").unwrap();

    // And finally we can call the wasm!
    hello.call(&mut store, ()).unwrap();
}
*/
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
