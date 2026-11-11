use alloc::collections::VecDeque;
use spin::Mutex;
use lazy_static::lazy_static;
use alloc::sync::Arc;
use crate::object::thread::ThreadObject;
use super::IdleThread;

lazy_static! {
    pub static ref SCHEDULER: Mutex<Scheduler> = Mutex::new(Scheduler::new());
}

pub struct Scheduler {
    threads: VecDeque<Arc<ThreadObject>>,
    processes: alloc::vec::Vec<Arc<crate::object::process::ProcessObject>>,
    current_thread: Option<Arc<ThreadObject>>,
    idle_thread: Option<Arc<ThreadObject>>,
    is_idle: bool,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            threads: VecDeque::new(),
            processes: alloc::vec::Vec::new(),
            current_thread: None,
            idle_thread: None,
            is_idle: false,
        }
    }

    pub fn set_idle_task(&mut self, thread: Arc<ThreadObject>) {
        self.idle_thread = Some(thread);
    }

    pub fn add_task(&mut self, thread: Arc<ThreadObject>) {
        self.threads.push_back(thread);
    }
    
    pub fn add_process(&mut self, process: Arc<crate::object::process::ProcessObject>) {
        self.processes.push(process);
    }
    
    pub fn get_process(&self, id: crate::object::process::ProcessId) -> Option<Arc<crate::object::process::ProcessObject>> {
        self.processes.iter().find(|p| p.id == id).cloned()
    }
    
    pub fn remove_process(&mut self, id: crate::object::process::ProcessId) {
        self.processes.retain(|p| p.id != id);
    }
    
    pub fn yield_current(&mut self) {
        if let Some(current) = self.current_thread.take() {
            // Check if process is Zombie/Crashed. If so, DO NOT push to threads queue
            let mut is_dead = false;
            if let Some(process) = current.process.upgrade() {
                let state = *process.state.lock();
                if state == crate::object::process::ProcessState::Zombie || state == crate::object::process::ProcessState::Crashed {
                    is_dead = true;
                }
            }
            if !is_dead {
                self.threads.push_back(current);
            }
        }
    }
}

pub fn add_task(thread: Arc<ThreadObject>) {
    SCHEDULER.lock().add_task(thread);
}

pub fn yield_current() {
    SCHEDULER.lock().yield_current();
}

pub fn schedule(current_rsp: u64) -> u64 {
    let mut scheduler = SCHEDULER.lock();
    
    // We must handle floating point context switch here as well.
    // If we have a current_task, save its FPU state.
    let current_thread_opt = scheduler.current_thread.take();
    if let Some(current) = &current_thread_opt {
        use super::ExtendedCpuState;
        let mut ctx = current.context.lock();
        ctx.fpu_state.save(); // Save FPU/SIMD state
        ctx.stack_ptr = current_rsp;
        drop(ctx);
        scheduler.threads.push_back(current.clone());
    }
    
    // Pick the next task
    while let Some(next_thread) = scheduler.threads.pop_front() {
        let mut is_dead = false;
        if let Some(process) = next_thread.process.upgrade() {
            let state = *process.state.lock();
            if state == crate::object::process::ProcessState::Zombie || state == crate::object::process::ProcessState::Crashed {
                is_dead = true;
            }
        }
        if is_dead {
            continue; // Skip dead threads
        }
        
        use super::ExtendedCpuState;
        let ctx = next_thread.context.lock();
        ctx.fpu_state.restore();
        let new_rsp = ctx.stack_ptr;
        drop(ctx);
        
        // If process changed, switch CR3
        if let Some(current) = &current_thread_opt {
            let next_pid = next_thread.process.upgrade().map(|p| p.id);
            let current_pid = current.process.upgrade().map(|p| p.id);
            if next_pid != current_pid {
                if let Some(next_proc) = next_thread.process.upgrade() {
                    unsafe { next_proc.address_space.switch(); }
                }
            }
        } else {
            // Context switching from nowhere (e.g. boot to first task)
            if let Some(next_proc) = next_thread.process.upgrade() {
                unsafe { next_proc.address_space.switch(); }
            }
        }
        
        // Update CpuLocal
        unsafe {
            let cpu_local = crate::arch::x86_64::cpu_local::get_cpu_local_mut();
            cpu_local.current_thread = Arc::as_ptr(&next_thread) as *mut _;
            if let Some(next_proc) = next_thread.process.upgrade() {
                cpu_local.current_process = Arc::as_ptr(&next_proc) as *mut _;
            } else {
                cpu_local.current_process = core::ptr::null_mut();
            }
        }

        scheduler.is_idle = false;
        scheduler.current_thread = Some(next_thread);
        return new_rsp;
    }
    
    // Queue is empty. Switch to IdleThread.
    if scheduler.is_idle {
        // Already running the idle task, nothing to do
        return current_rsp;
    }

        scheduler.is_idle = true;
        if let Some(idle) = scheduler.idle_thread.take() {
            use super::ExtendedCpuState;
            let ctx = idle.context.lock();
            ctx.fpu_state.restore();
            let new_rsp = ctx.stack_ptr;
            drop(ctx);
            // Put it back
            scheduler.idle_thread = Some(idle);
            new_rsp
        } else {
            panic!("Queue is empty, IdleThread stack not yet allocated!");
        }
    }
