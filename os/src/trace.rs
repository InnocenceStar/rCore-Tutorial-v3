//! Kernel trace integration.

#![allow(static_mut_refs)]

use crate::config::CLOCK_FREQ;
use crate::sync::UPSafeCell;
use crate::task::TaskControlBlock;
use crate::timer;
use alloc::string::String;
use alloc::sync::Arc;
use lazy_static::*;
use ostrace::{BufferMode, OffCpuState, TaskRef, TraceImage, TraceImageConfig, TracePlatform};

const MAX_CPUS: usize = 1;
const TRACE_IMAGE_SIZE: usize = 16 * 1024 * 1024;
const TRACE_PER_CPU_BUFFER_SIZE: usize = TRACE_IMAGE_SIZE - 4096;

/// State entered by the task that is leaving the CPU.
#[derive(Clone, Copy)]
pub enum TraceOffCpuState {
    /// The task remains runnable.
    Running,
    /// The task has exited.
    Dead,
}

impl TraceOffCpuState {
    fn to_ostrace(self) -> OffCpuState {
        match self {
            Self::Running => OffCpuState::Running,
            Self::Dead => OffCpuState::Dead,
        }
    }
}

struct KernelTracePlatform;

impl TracePlatform for KernelTracePlatform {
    fn now_ns(&self) -> u64 {
        ((timer::get_time() as u128 * 1_000_000_000u128) / CLOCK_FREQ as u128) as u64
    }

    fn cpu_id(&self) -> u32 {
        0
    }
}

#[derive(Clone)]
struct TraceTaskInfo {
    pid: u32,
    comm: String,
}

impl TraceTaskInfo {
    fn idle() -> Self {
        Self {
            pid: 0,
            comm: String::from("idle"),
        }
    }

    fn as_task_ref(&self) -> TaskRef<'_> {
        TaskRef {
            comm: self.comm.as_str(),
            tid: self.pid,
            tgid: self.pid,
            prio: 0,
        }
    }
}

#[derive(Clone)]
struct PendingSwitch {
    prev: TraceTaskInfo,
    state: TraceOffCpuState,
}

lazy_static! {
    static ref PENDING_SWITCH: UPSafeCell<Option<PendingSwitch>> = unsafe { UPSafeCell::new(None) };
}

#[unsafe(link_section = ".trace.image")]
static mut TRACE_IMAGE_BYTES: [u8; TRACE_IMAGE_SIZE] = [0; TRACE_IMAGE_SIZE];
static mut TRACE_IMAGE: Option<TraceImage<'static, KernelTracePlatform>> = None;

/// Starts the single kernel trace image session.
pub fn init() {
    unsafe {
        let image = TraceImage::init(TraceImageConfig {
            bytes: &mut TRACE_IMAGE_BYTES,
            cpu_count: MAX_CPUS,
            per_cpu_buffer_size: TRACE_PER_CPU_BUFFER_SIZE,
            mode: BufferMode::Overwrite,
            platform: KernelTracePlatform,
        })
        .expect("failed to initialize trace image");
        TRACE_IMAGE = Some(image);
    }
}

/// Finishes the active trace image session.
pub fn finish() {
    unsafe {
        if let Some(image) = TRACE_IMAGE.as_mut() {
            image.finish();
        }
        TRACE_IMAGE = None;
    }
}

/// Records the current task as the next scheduler switch source.
pub fn set_pending_switch(task: &Arc<TaskControlBlock>, state: TraceOffCpuState) {
    let (pid, comm) = task.trace_identity();
    *PENDING_SWITCH.exclusive_access() = Some(PendingSwitch {
        prev: TraceTaskInfo {
            pid: pid as u32,
            comm,
        },
        state,
    });
}

/// Records a sched_switch event to `next`.
pub fn record_sched_switch_to(next: &Arc<TaskControlBlock>) {
    let pending = PENDING_SWITCH
        .exclusive_access()
        .take()
        .unwrap_or(PendingSwitch {
            prev: TraceTaskInfo::idle(),
            state: TraceOffCpuState::Running,
        });
    let (pid, comm) = next.trace_identity();
    let next = TraceTaskInfo {
        pid: pid as u32,
        comm,
    };
    record_sched_switch(&pending.prev, pending.state, &next);
}

fn record_sched_switch(prev: &TraceTaskInfo, prev_state: TraceOffCpuState, next: &TraceTaskInfo) {
    unsafe {
        if let Some(image) = TRACE_IMAGE.as_mut() {
            let _ = image.context_switch(
                prev.as_task_ref(),
                prev_state.to_ostrace(),
                next.as_task_ref(),
            );
        }
    }
}
