use x86_64::registers::model_specific::{GsBase, KernelGsBase};
use x86_64::VirtAddr;

#[repr(C)]
pub struct CpuLocal {
    pub cpu_id: u32,
    pub current_thread: *mut crate::object::thread::ThreadObject,
    pub current_process: *mut crate::object::process::ProcessObject,
    pub kernel_stack_top: u64,
    pub scheduler_data: u64,  // *mut SchedulerCpuState later
    pub syscall_count: u64,
    pub interrupt_depth: u32,
}

static mut CPU0_LOCAL: CpuLocal = CpuLocal {
    cpu_id: 0,
    current_thread: core::ptr::null_mut(),
    current_process: core::ptr::null_mut(),
    kernel_stack_top: 0,
    scheduler_data: 0,
    syscall_count: 0,
    interrupt_depth: 0,
};

pub fn init() {
    unsafe {
        // We write the address of CPU0_LOCAL to KernelGsBase.
        // During syscall, `swapgs` will place this into `GsBase`, allowing access via `gs:[offset]`.
        // During normal kernel execution, `GsBase` holds it. Wait.
        // Actually, normally `GsBase` points to CpuLocal while in kernel, and `KernelGsBase` holds the user's GS base.
        // On entry from user mode, `swapgs` swaps them, putting CpuLocal into `GsBase`.
        // So we should set `GsBase` to CpuLocal now, since we are in the kernel.
        // `KernelGsBase` will be set to 0 (user's GS base).
        
        let cpu_local_ptr = &raw const CPU0_LOCAL as u64;
        GsBase::write(VirtAddr::new(cpu_local_ptr));
        KernelGsBase::write(VirtAddr::new(0)); // User GS base is initially 0
    }
}

pub unsafe fn get_cpu_local_mut() -> &'static mut CpuLocal {
    &mut CPU0_LOCAL
}
