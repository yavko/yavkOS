use crate::{gdt, hlt_loop, println, serial_println};
use lazy_static::lazy_static;
use paste::paste;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;
pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

macro_rules! ehand {
    ($name:ident) => {
        paste! {
            extern "x86-interrupt" fn [<$name _handler>](stackframe: InterruptStackFrame) {
                println!("EXCEPTION: {}:\n{:#?}", stringify!($name:upper), stackframe);
                serial_println!("EXCEPTION: {}:\n{:#?}", stringify!($name:upper), stackframe);
            }
        }
    };
    (code $name:ident) => {
        paste! {
            extern "x86-interrupt" fn [<$name _handler>](stackframe: InterruptStackFrame, ecode: u64) {
                println!("EXCEPTION: {} ({}):\n{:#?}", stringify!([<$name:upper>]).replace("_", " "), ecode, stackframe);
                serial_println!("EXCEPTION: {} ({}):\n{:#?}", stringify!([<$name:upper>]).replace("_", " "), ecode, stackframe);
            }
        }
    };
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        unsafe {
            idt.page_fault
                .set_handler_fn(page_fault_handler)
                .set_stack_index(gdt::PAGE_FAULT_IST_INDEX);
        }
        ehand!(division);
        idt.divide_error.set_handler_fn(division_handler);
        ehand!(bound_range);
        idt.bound_range_exceeded.set_handler_fn(bound_range_handler);
        ehand!(invalid_opcode);
        idt.invalid_opcode.set_handler_fn(invalid_opcode_handler);
        ehand!(device_not_available);
        idt.device_not_available
            .set_handler_fn(device_not_available_handler);
        ehand!(code invalid_tss);
        idt.invalid_tss.set_handler_fn(invalid_tss_handler);
        ehand!(code segment_not_present);
        idt.segment_not_present
            .set_handler_fn(segment_not_present_handler);
        ehand!(code stack_seg_fault);
        idt.stack_segment_fault
            .set_handler_fn(stack_seg_fault_handler);
        ehand!(code general_protection_fault);
        unsafe {
            idt.general_protection_fault
                .set_handler_fn(general_protection_fault_handler)
                .set_stack_index(gdt::GENERAL_PROTECTION_FAULT_IST_INDEX);
        }
        ehand!(x87_floating_point_exception);
        idt.x87_floating_point
            .set_handler_fn(x87_floating_point_exception_handler);
        ehand!(simd_floating_point_exception);
        idt.simd_floating_point
            .set_handler_fn(simd_floating_point_exception_handler);

        idt[InterruptIndex::Timer.as_usize()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_usize()].set_handler_fn(keyboard_interrupt_handler);
        idt[InterruptIndex::Mouse.as_usize()].set_handler_fn(mouse_interrupt_handler);
        idt
    };
}

pub fn init_idt() {
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stackframe: InterruptStackFrame) {
    println!("EXCEPTION: BREAKPOINT:\n{:#?}", stackframe);
}

extern "x86-interrupt" fn double_fault_handler(
    stackframe: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    panic!("EXCEPTION: DOUBLE FAULT:\n{:#?}", stackframe);
}

extern "x86-interrupt" fn page_fault_handler(
    stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;

    println!("EXCEPTION: PAGE FAULT");
    println!("Accessed Address: {:?}", Cr2::read());
    println!("Error Code: {:?}", error_code);
    println!("{:#?}", stack_frame);
    hlt_loop();
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    //println!(".");
    //serial_println!("running...");
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1};
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::Ignore
            ));
    }

    let mut port = Port::new(0x60);
    let scancode: u8 = unsafe { port.read() };
    crate::task::keyboard::add_scancode(scancode);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}

extern "x86-interrupt" fn mouse_interrupt_handler(_stack_frame: InterruptStackFrame) {
    use x86_64::instructions::port::PortReadOnly;

    let mut port = PortReadOnly::new(0x60);
    let packet = unsafe { port.read() };
    crate::task::mouse::write(packet);

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Mouse.as_u8());
    }
}

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
    Mouse = PIC_1_OFFSET + 12,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
}

#[test_case]
fn test_breakpoint_exception() {
    // invoke a breakpoint exception
    x86_64::instructions::interrupts::int3();
}
