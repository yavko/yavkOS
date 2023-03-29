use lazy_static::lazy_static;
use x86_64::structures::gdt::{Descriptor, GlobalDescriptorTable, SegmentSelector};
use x86_64::structures::tss::TaskStateSegment;
use x86_64::VirtAddr;

const STACK_SIZE: usize = 4096 * 5;
pub const DOUBLE_FAULT_IST_INDEX: u16 = 0;
pub const PAGE_FAULT_IST_INDEX: u16 = 1;
pub const GENERAL_PROTECTION_FAULT_IST_INDEX: u16 = 2;

// some code from https://github.com/vinc/moros/blob/trunk/src/sys/gdt.rs

lazy_static! {
    static ref TSS: TaskStateSegment = {
        let mut tss = TaskStateSegment::new();
        tss.privilege_stack_table[0] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            VirtAddr::from_ptr(unsafe { &STACK }) + STACK_SIZE
        };
        tss.interrupt_stack_table[DOUBLE_FAULT_IST_INDEX as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];

            let stack_start = VirtAddr::from_ptr(unsafe { &STACK });
            stack_start + STACK_SIZE
        };
        tss.interrupt_stack_table[PAGE_FAULT_IST_INDEX as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            VirtAddr::from_ptr(unsafe { &STACK }) + STACK_SIZE
        };
        tss.interrupt_stack_table[GENERAL_PROTECTION_FAULT_IST_INDEX as usize] = {
            static mut STACK: [u8; STACK_SIZE] = [0; STACK_SIZE];
            VirtAddr::from_ptr(unsafe { &STACK }) + STACK_SIZE
        };
        tss
    };
    static ref GDT: (GlobalDescriptorTable, Selectors) = {
        let mut gdt = GlobalDescriptorTable::new();
        let code = gdt.add_entry(Descriptor::kernel_code_segment());
        let tss = gdt.add_entry(Descriptor::tss_segment(&TSS));
        let data = gdt.add_entry(Descriptor::kernel_data_segment());
        let user_code = gdt.add_entry(Descriptor::user_code_segment());
        let user_data = gdt.add_entry(Descriptor::user_data_segment());

        (
            gdt,
            Selectors {
                code,
                tss,
                data,
                user_code,
                user_data,
            },
        )
    };
}

struct Selectors {
    code: SegmentSelector,
    tss: SegmentSelector,
    data: SegmentSelector,
    pub user_code: SegmentSelector,
    pub user_data: SegmentSelector,
}

pub fn init() {
    use x86_64::instructions::segmentation::{Segment, CS, DS, SS};
    use x86_64::instructions::tables::load_tss;

    GDT.0.load();
    unsafe {
        CS::set_reg(GDT.1.code);
        DS::set_reg(GDT.1.data);
        SS::set_reg(SegmentSelector(0));
        load_tss(GDT.1.tss);
    }
}
