pub mod gdt;

use lazy_static::lazy_static;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame, PageFaultErrorCode};
use x86_64::registers::control::Cr2;
use pic8259::ChainedPics;
use spin;
use crate::println;

pub const PIC_1_OFFSET: u8 = 32;
pub const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: spin::Mutex<ChainedPics> =
    spin::Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
pub enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }

    fn as_usize(self) -> usize {
        usize::from(self.as_u8())
    }
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
        idt.page_fault.set_handler_fn(page_fault_handler);
        
        extern "C" {
            fn timer_interrupt_stub();
        }
        
        unsafe {
            idt[InterruptIndex::Timer.as_u8()]
                .set_handler_addr(x86_64::VirtAddr::new(timer_interrupt_stub as u64));
        }
        
        idt
    };
}

pub fn init() {
    gdt::init();
    IDT.load();
    unsafe { PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

extern "x86-interrupt" fn breakpoint_handler(_stack_frame: InterruptStackFrame) {
    // Breakpoint exception
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    println!("{:#?}", stack_frame);
    panic!("EXCEPTION: DOUBLE FAULT");
}

extern "x86-interrupt" fn page_fault_handler(
    interrupt_stack_frame: InterruptStackFrame,
    error_code: PageFaultErrorCode,
) {
    use x86_64::registers::control::Cr2;
    // x86_64 crate currently doesn't easily expose the CS register from the stack frame
    // without reading the raw frame, but we can assume if it's a Ring 3 segment, it's user.
    // Instead of parsing CS, we can just kill the current process if one exists!
    // A kernel page fault shouldn't happen unless we are doing user_copy, which we handle.
    
    let cpu_local = unsafe { crate::arch::x86_64::cpu_local::get_cpu_local_mut() };
    if !cpu_local.current_process.is_null() {
        crate::println!("USER EXCEPTION: PAGE FAULT at {:?}", Cr2::read());
        let process = unsafe { &*cpu_local.current_process };
        *process.state.lock() = crate::object::process::ProcessState::Crashed;
        *process.exit_info.lock() = Some(crate::object::process::ProcessExit {
            state: crate::object::process::ProcessState::Crashed,
            code: -11, // SIGSEGV
        });
        
        unsafe {
            core::arch::asm!(
                "sti",
                "2:",
                "hlt",
                "jmp 2b",
            );
        }
    } else {
        crate::println!("KERNEL EXCEPTION: PAGE FAULT");
        crate::println!("Accessed Address: {:?}", Cr2::read());
        crate::println!("Error Code: {:?}", error_code);
        crate::println!("{:#?}", interrupt_stack_frame);
        loop {
            x86_64::instructions::hlt();
        }
    }
}

#[no_mangle]
pub extern "C" fn timer_interrupt_handler_rust(rsp: u64) -> u64 {
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
    
    // Call scheduler and get the (possibly new) stack pointer
    crate::task::scheduler::schedule(rsp)
}
