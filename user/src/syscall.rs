use core::arch::asm;

const SYSCALL_WRITE: usize = 64;
const SYSCALL_EXIT: usize = 93;
/// 系统调用
/// ## ecall
/// - 保存断点: 将当前pc(ecall的下一条指令的地址)保存到sepc寄存器, 方便内核处理完成回来;
/// - 切换特权级: 将当前User特权保存到sstatus.SPP, 然后切换到Supervisor;
/// - 跳转入口: 读取stvec寄存器的值, 跳转到该地址执行;
fn syscall(id: usize, args: [usize; 3]) -> isize {
    let mut ret: isize;
    unsafe {
        asm!(
            "ecall",
            inlateout("x10") args[0] => ret,
            in("x11") args[1],
            in("x12") args[2],
            in("x17") id
        );
    }
    ret
}

pub fn sys_write(fd: usize, buffer: &[u8]) -> isize {
    syscall(SYSCALL_WRITE, [fd, buffer.as_ptr() as usize, buffer.len()])
}

pub fn sys_exit(exit_code: i32) -> isize {
    syscall(SYSCALL_EXIT, [exit_code as usize, 0, 0])
}
