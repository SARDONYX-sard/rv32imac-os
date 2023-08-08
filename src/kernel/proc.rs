use core::{
    arch::asm,
    sync::atomic::{AtomicBool, AtomicUsize, Ordering},
};

// const PROCS_MAX: usize = 8;
const PROCS_MAX: usize = 3;
// const PROC_STACK_LEN: usize = 4096;
const PROC_STACK_LEN: usize = 4096 / 2;
static IS_SET_RUNNER: AtomicBool = AtomicBool::new(false);
static PROC_RUNNER_PTR: AtomicUsize = AtomicUsize::new(0);

/// Exists only to obtain the return execution address of the context switch
/// and register it in the `return argument` register.
/// (exclude 0 pid)
fn recycle_and_run_next() {
    unsafe { (*Executer::as_mut_ptr()).t_return() };
}
pub fn run_next_proc() {
    unsafe { (*Executer::as_mut_ptr()).run_next() };
}

/// Process Runner
#[derive(Debug)]
pub struct Executer {
    procs: [Process; PROCS_MAX],
    running_proc_idx: usize,
}

impl Default for Executer {
    fn default() -> Self {
        Self::new()
    }
}

impl Executer {
    /// Init proc queue.
    pub fn new() -> Self {
        Self {
            procs: [
                Process {
                    pid: 0,
                    state: ProcState::Running,
                    ..Default::default()
                },
                Process::new(1),
                Process::new(2),
                // Process::new(4),
                // Process::new(5),
                // Process::new(6),
                // Process::new(7),
                // Process::new(8),
            ],
            running_proc_idx: 0,
        }
    }

    /// Push task to task queue.
    pub fn push(&mut self, task_ptr: fn()) {
        let unused_proc = self
            .procs
            .iter_mut()
            .find(|proc| proc.state == ProcState::Unused)
            .expect("no free process slots");

        unsafe {
            // calculate stack end field but it's stack start.
            let stack_start_ptr = unused_proc.stack.as_mut_ptr().add(unused_proc.stack.len());
            // Allocate more until the 8-byte alignment requirement is met. (by ABI)
            // Using the fact that multiplying by 8 is equivalent to meaning that all bits of 8-1 will be 0.
            let stack_start_ptr = (stack_start_ptr as usize & !(8 - 1)) as *mut u8;
            // unused_proc.ctx.ra = recycle_and_run_next as usize;
            unused_proc.ctx.ra = recycle_and_run_next as usize;
            unused_proc.ctx.current_pc = task_ptr as usize;
            unused_proc.ctx.sp = stack_start_ptr.sub(32) as usize;
        }
        unused_proc.state = ProcState::Runnable;
    }

    /// Execute the pushed tasks in order.
    pub fn run(&mut self) {
        // Set once RUNNER ptr.
        if !IS_SET_RUNNER.load(Ordering::Acquire) {
            PROC_RUNNER_PTR.store(self as *const Executer as usize, Ordering::Release);
            IS_SET_RUNNER.store(false, Ordering::Release);
        }
        while self.run_next() {}
    }

    /// Recycle completed proc & run other proc.
    /// - This function is intended to be called after task completion.
    pub(self) fn t_return(&mut self) {
        if self.running_proc_idx != 0 {
            self.procs[self.running_proc_idx].state = ProcState::Unused;
            self.run_next();
        }
    }

    /// Yield process.
    ///
    /// # Return
    /// Are there any tasks remaining? true/false
    pub(self) fn run_next(&mut self) -> bool {
        // 1. search runnable proc. nothing? => return false
        let mut next_idx = self.running_proc_idx;
        while self.procs[next_idx].state != ProcState::Runnable {
            // if index max, fallback to 0.
            next_idx = (next_idx + 1) % self.procs.len();
            if next_idx == self.running_proc_idx {
                return false; // Runnable processes is nothing.
            }
        }

        // 2. change tasks state: prev = Runnable, next = Running
        if self.procs[self.running_proc_idx].state != ProcState::Unused {
            self.procs[self.running_proc_idx].state = ProcState::Runnable;
        }
        self.procs[next_idx].state = ProcState::Running;
        let prev = self.running_proc_idx;
        self.running_proc_idx = next_idx;

        // 3. As you can see
        unsafe { switch_context(&mut self.procs[prev].ctx, &self.procs[next_idx].ctx) };
        // NOTE!: The process with pid 0 does nothing and returns to the address after the context switch.
        true
    }

    unsafe fn as_mut_ptr() -> *mut Executer {
        let runner_ptr = PROC_RUNNER_PTR.load(Ordering::Acquire) as *mut Executer;
        assert_ne!(
            runner_ptr as usize, 0,
            "ProcRunner may not have been initialized. runner_ptr: {}",
            runner_ptr as usize
        );
        runner_ptr
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum ProcState {
    Unused,
    Running,
    Runnable,
}

/// # PCB: Process Control Block)
#[derive(Clone, Debug)]
pub struct Process {
    #[allow(unused)]
    pid: usize,
    state: ProcState,
    #[allow(unused)]
    page_table: usize,
    /// ## This stack starts the last index as usual.
    ///
    /// one process's kernel stack(8192 == 8KiB)
    /// - CPU registers
    /// - Return destination of the function
    /// - Local variables in each function
    /// - Others.
    stack: [u8; PROC_STACK_LEN],
    ctx: ProcContext,
}

impl Default for Process {
    fn default() -> Self {
        Self {
            pid: Default::default(),
            state: ProcState::Unused,
            page_table: Default::default(),
            stack: [0; PROC_STACK_LEN],
            ctx: Default::default(),
        }
    }
}

impl Process {
    fn new(pid: usize) -> Self {
        Self {
            pid,
            state: ProcState::Unused,
            page_table: 1,
            stack: [0; PROC_STACK_LEN],
            ctx: ProcContext::new(),
        }
    }
}

/// Saves registers that should be saved by the caller as specified in the ABI calling convention.
#[derive(Clone, Debug, Default)]
#[repr(C)]
pub struct ProcContext {
    ra: usize,
    /// stack pointer
    sp: usize,
    s0: usize,
    s1: usize,
    s2: usize,
    s3: usize,
    s4: usize,
    s5: usize,
    s6: usize,
    s7: usize,
    s8: usize,
    s9: usize,
    s10: usize,
    s11: usize,
    /// next task ptr
    current_pc: usize,
}

impl ProcContext {
    fn new() -> Self {
        Self {
            ra: 0,
            sp: 0,
            s0: 0,
            s1: 0,
            s2: 0,
            s3: 0,
            s4: 0,
            s5: 0,
            s6: 0,
            s7: 0,
            s8: 0,
            s9: 0,
            s10: 0,
            s11: 0,
            current_pc: 0,
        }
    }
}

/// Saves registers that should be saved by the caller as specified in the ABI calling convention.
///
/// # parameters
/// - prev_sp: Stack pointer of previous process
/// - next_sp: Stack pointer of next process
#[naked]
#[no_mangle]
unsafe extern "C" fn switch_context(prev_ctx: *mut ProcContext, next_ctx: *const ProcContext) {
    // NOTE: a0 = prev_sp, a1 = next
    // sw reg, mem: mem <- reg
    // lw reg, mem: reg = mem
    // ret: => jalr x0, ra, 0 jump to return address
    unsafe {
        asm!(
            "
            sw  ra,  0 * 4(a0)
            sw  sp,  1 * 4(a0)
            sw  s0,  2 * 4(a0)
            sw  s1,  3 * 4(a0)
            sw  s2,  4 * 4(a0)
            sw  s3,  5 * 4(a0)
            sw  s4,  6 * 4(a0)
            sw  s5,  7 * 4(a0)
            sw  s6,  8 * 4(a0)
            sw  s7,  9 * 4(a0)
            sw  s8, 10 * 4(a0)
            sw  s9, 11 * 4(a0)
            sw s10, 12 * 4(a0)
            sw s11, 13 * 4(a0)
            sw  ra, 14 * 4(a0)

            lw  ra,  0 * 4(a1)
            lw  sp,  1 * 4(a1)
            lw  s0,  2 * 4(a1)
            lw  s1,  3 * 4(a1)
            lw  s2,  4 * 4(a1)
            lw  s3,  5 * 4(a1)
            lw  s4,  6 * 4(a1)
            lw  s5,  7 * 4(a1)
            lw  s6,  8 * 4(a1)
            lw  s7,  9 * 4(a1)
            lw  s8, 10 * 4(a1)
            lw  s9, 11 * 4(a1)
            lw s10, 12 * 4(a1)
            lw s11, 13 * 4(a1)
            lw  t0, 14 * 4(a1)
            jr  t0",
            options(noreturn)
        );
    }
}
