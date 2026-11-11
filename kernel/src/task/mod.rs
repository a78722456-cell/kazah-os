pub mod scheduler;

pub trait ExtendedCpuState {
    fn save(&mut self);
    fn restore(&self);
}

pub struct IdleThread {
    pub cpu_id: usize,
}

impl IdleThread {
    pub extern "C" fn run() -> ! {
        loop {
            x86_64::instructions::hlt();
        }
    }
}

use core::sync::atomic::{AtomicU64, Ordering};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskId(u64);

#[repr(align(64))]
pub struct FpuState(pub [u8; 1024]);

impl TaskId {
    pub fn new() -> Self {
        static NEXT_ID: AtomicU64 = AtomicU64::new(0);
        TaskId(NEXT_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl ExtendedCpuState for FpuState {
    fn save(&mut self) {
        unsafe {
            core::arch::asm!(
                "xsave [{}]",
                in(reg) self.0.as_mut_ptr(),
                in("eax") 7u32,
                in("edx") 0u32,
            );
        }
    }

    fn restore(&self) {
        unsafe {
            core::arch::asm!(
                "xrstor [{}]",
                in(reg) self.0.as_ptr(),
                in("eax") 7u32,
                in("edx") 0u32,
            );
        }
    }
}

// Global assembly for timer interrupt stub
core::arch::global_asm!(
    r#"
    .global timer_interrupt_stub
    timer_interrupt_stub:
        // CPU pushed: SS, RSP, RFLAGS, CS, RIP
        push r15
        push r14
        push r13
        push r12
        push r11
        push r10
        push r9
        push r8
        push rbp
        push rdi
        push rsi
        push rdx
        push rcx
        push rbx
        push rax

        // Pass the stack pointer (RSP) as the first argument (RDI)
        mov rdi, rsp

        // Call Rust handler
        call timer_interrupt_handler_rust

        // The Rust handler returns the new RSP in RAX (C calling convention)
        mov rsp, rax

        // Pop the TrapFrame
        pop rax
        pop rbx
        pop rcx
        pop rdx
        pop rsi
        pop rdi
        pop rbp
        pop r8
        pop r9
        pop r10
        pop r11
        pop r12
        pop r13
        pop r14
        pop r15

        iretq
    "#
);
