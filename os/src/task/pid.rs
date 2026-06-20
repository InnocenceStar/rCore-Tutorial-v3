//!Implementation of [`PidAllocator`] and kernel stack slot allocation.
use crate::config::{KERNEL_STACK_SIZE, PAGE_SIZE, TRAMPOLINE};
use crate::mm::{KERNEL_SPACE, MapPermission, VirtAddr};
use crate::sync::UPSafeCell;
use alloc::vec::Vec;
use lazy_static::*;

///Pid Allocator struct.
pub struct PidAllocator {
    current: usize,
}

impl PidAllocator {
    ///Create an empty `PidAllocator`.
    pub fn new() -> Self {
        PidAllocator { current: 0 }
    }

    ///Allocate a monotonically increasing pid.
    pub fn alloc(&mut self) -> usize {
        let pid = self.current;
        self.current += 1;
        pid
    }
}

lazy_static! {
    pub static ref PID_ALLOCATOR: UPSafeCell<PidAllocator> =
        unsafe { UPSafeCell::new(PidAllocator::new()) };
    static ref KSTACK_ALLOCATOR: UPSafeCell<KStackAllocator> =
        unsafe { UPSafeCell::new(KStackAllocator::new()) };
}

///Allocate a pid from PID_ALLOCATOR.
pub fn pid_alloc() -> usize {
    PID_ALLOCATOR.exclusive_access().alloc()
}

struct KStackAllocator {
    current: usize,
    recycled: Vec<usize>,
}

impl KStackAllocator {
    fn new() -> Self {
        KStackAllocator {
            current: 0,
            recycled: Vec::new(),
        }
    }

    fn alloc(&mut self) -> KStackSlotId {
        if let Some(kstack_slot_id) = self.recycled.pop() {
            KStackSlotId(kstack_slot_id)
        } else {
            let kstack_slot_id = self.current;
            self.current += 1;
            KStackSlotId(kstack_slot_id)
        }
    }

    fn dealloc(&mut self, kstack_slot_id: usize) {
        assert!(kstack_slot_id < self.current);
        assert!(
            !self.recycled.iter().any(|id| *id == kstack_slot_id),
            "kernel stack slot id {} has been deallocated!",
            kstack_slot_id
        );
        self.recycled.push(kstack_slot_id);
    }
}

pub struct KStackSlotId(pub usize);

impl Drop for KStackSlotId {
    fn drop(&mut self) {
        KSTACK_ALLOCATOR.exclusive_access().dealloc(self.0);
    }
}

fn kstack_alloc() -> KStackSlotId {
    KSTACK_ALLOCATOR.exclusive_access().alloc()
}

/// Return (bottom, top) of a kernel stack in kernel space.
pub fn kernel_stack_position(kstack_slot_id: usize) -> (usize, usize) {
    let top = TRAMPOLINE - kstack_slot_id * (KERNEL_STACK_SIZE + PAGE_SIZE);
    let bottom = top - KERNEL_STACK_SIZE;
    (bottom, top)
}

///Kernelstack for app.
pub struct KernelStack {
    kstack_slot_id: KStackSlotId,
}

impl KernelStack {
    ///Create a kernelstack from a kernel stack slot.
    pub fn new() -> Self {
        let kstack_slot_id = kstack_alloc();
        let (kernel_stack_bottom, kernel_stack_top) = kernel_stack_position(kstack_slot_id.0);
        KERNEL_SPACE.exclusive_access().insert_framed_area(
            kernel_stack_bottom.into(),
            kernel_stack_top.into(),
            MapPermission::R | MapPermission::W,
        );
        KernelStack { kstack_slot_id }
    }

    #[allow(unused)]
    ///Push a value on top of kernelstack.
    pub fn push_on_top<T>(&self, value: T) -> *mut T
    where
        T: Sized,
    {
        let kernel_stack_top = self.get_top();
        let ptr_mut = (kernel_stack_top - core::mem::size_of::<T>()) as *mut T;
        unsafe {
            *ptr_mut = value;
        }
        ptr_mut
    }

    ///Get the value on the top of kernelstack.
    pub fn get_top(&self) -> usize {
        let (_, kernel_stack_top) = kernel_stack_position(self.kstack_slot_id.0);
        kernel_stack_top
    }
}

impl Drop for KernelStack {
    fn drop(&mut self) {
        let (kernel_stack_bottom, _) = kernel_stack_position(self.kstack_slot_id.0);
        let kernel_stack_bottom_va: VirtAddr = kernel_stack_bottom.into();
        KERNEL_SPACE
            .exclusive_access()
            .remove_area_with_start_vpn(kernel_stack_bottom_va.into());
    }
}
