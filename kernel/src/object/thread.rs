use super::{ObjectHeader, ObjectType, KernelObject};
use crate::task::{TaskId, FpuState};
use crate::object::process::ProcessObject;
use spin::Mutex;
use alloc::sync::Weak;

pub struct ThreadContext {
    pub stack_ptr: u64,
    pub fpu_state: FpuState,
}

pub struct ThreadObject {
    pub header: ObjectHeader,
    pub id: TaskId,
    pub process: Weak<ProcessObject>,
    pub context: Mutex<ThreadContext>,
}

impl KernelObject for ThreadObject {
    fn header(&self) -> &ObjectHeader {
        &self.header
    }
    fn as_any(&self) -> &dyn core::any::Any {
        self
    }
}

impl ThreadObject {
    pub fn new(entry_point: u64, process: Weak<ProcessObject>) -> Self {
        let id = TaskId::new();
        // Allocate a 16KB stack for the task from the heap
        let stack_size = 16 * 1024;
        let stack = alloc::vec::Vec::<u8>::with_capacity(stack_size);
        let stack_end = stack.as_ptr() as u64 + stack_size as u64;

        // Forget the vector so its memory isn't freed
        core::mem::forget(stack);

        let mut rsp = stack_end;
        
        unsafe {
            // CPU Trap Frame
            rsp -= 8; *(rsp as *mut u64) = 0x10; // SS (Data segment)
            rsp -= 8; *(rsp as *mut u64) = stack_end; // RSP
            rsp -= 8; *(rsp as *mut u64) = 0x202; // RFLAGS (Interrupts enabled)
            rsp -= 8; *(rsp as *mut u64) = 0x08; // CS (Code segment)
            rsp -= 8; *(rsp as *mut u64) = entry_point; // RIP
            
            // General Purpose Registers
            for _ in 0..15 {
                rsp -= 8;
                *(rsp as *mut u64) = 0;
            }
        }

        Self {
            header: ObjectHeader::new(ObjectType::Thread),
            id,
            process,
            context: Mutex::new(ThreadContext {
                stack_ptr: rsp,
                fpu_state: FpuState([0; 1024]),
            }),
        }
    }

    pub fn new_user(entry_point: u64, process: Weak<ProcessObject>) -> Self {
        let id = TaskId::new();
        // Allocate a 16KB stack for the task from the heap
        let stack_size = 16 * 1024;
        let stack = alloc::vec::Vec::<u8>::with_capacity(stack_size);
        let stack_end = stack.as_ptr() as u64 + stack_size as u64;

        // Forget the vector so its memory isn't freed
        core::mem::forget(stack);

        let mut rsp = stack_end;
        
        unsafe {
            // CPU Trap Frame (User Space)
            rsp -= 8; *(rsp as *mut u64) = 0x1B; // SS (User Data segment RPL=3)
            rsp -= 8; *(rsp as *mut u64) = stack_end; // RSP
            rsp -= 8; *(rsp as *mut u64) = 0x202; // RFLAGS (Interrupts enabled)
            rsp -= 8; *(rsp as *mut u64) = 0x23; // CS (User Code segment RPL=3)
            rsp -= 8; *(rsp as *mut u64) = entry_point; // RIP
            
            // General Purpose Registers
            for _ in 0..15 {
                rsp -= 8;
                *(rsp as *mut u64) = 0;
            }
        }

        Self {
            header: ObjectHeader::new(ObjectType::Thread),
            id,
            process,
            context: Mutex::new(ThreadContext {
                stack_ptr: rsp,
                fpu_state: FpuState([0; 1024]),
            }),
        }
    }
}
