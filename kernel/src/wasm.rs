use kernel::println;
//use wasmi::{Engine, Module};
use wasmi::*;

pub fn read_wasm_string(offset: u32, length: u32, wasm_mem: &[u8]) -> &str {
    ::core::str::from_utf8(&wasm_mem[offset as usize..offset as usize + length as usize])
        .expect("read_wasm_cstring failed to parse invalid utf-8 string")
}

pub async fn example_exec() {
    // First step is to create the Wasm execution engine with some config.
    // In this example we are using the default configuration.
    let engine = Engine::default();
    let wasm = include_bytes!("../../test.wasm");
    let module = Module::new(&engine, &wasm[..]).unwrap();

    // All Wasm objects operate within the context of a `Store`.
    // Each `Store` has a type parameter to store host-specific data,
    // which in this case we are using `42` for.
    type HostState = u32;

    let mut store = Store::new(&engine, 42);
    let host_hello = Func::wrap(&mut store, |caller: Caller<'_, HostState>, param: i32| {
        println!("Got {} from WebAssembly", param);
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
    let instance = linker
        .instantiate(&mut store, &module)
        .unwrap()
        .start(&mut store)
        .unwrap();
    let hello = instance.get_typed_func::<(), ()>(&store, "hello").unwrap();

    // And finally we can call the wasm!
    hello.call(&mut store, ()).unwrap();
}
