.altmacro
.macro SAVE_GP n
    sd x\n, \n*8(sp)
.endm
.macro LOAD_GP n
    ld x\n, \n*8(sp)
.endm
    # 保存用户态寄存器到TrapContext, 切换到内核栈，跳转trap_handler
    .section .text.trampoline
    .globl __alltraps
    .globl __restore
    # `.align 2`：只是为了满足编译器/汇编器的语法检查，确保指令本身是 4 字节对齐的。因为此时已经满足 4K 对齐，所以它实际上不会产生任何填充。
    # TRAMPOLINE 必须严格等于 __alltraps 的地址。哪怕差一个字节，导致 CPU 先执行了填充物，整个“切换栈 -> 保存现场”的精密链条就会断裂，导致系统崩溃。
    .align 2
__alltraps:
    # 切换栈:user->kernel; 切换前，sp指向用户栈，sscratch 则被设置为指向应用地址空间中存放 Trap 上下文的位置
    csrrw sp, sscratch, sp
    # 切换后, sp 指向trap上下文用于保存一些寄存器(全程在用户空间完成保存)
    # now sp->*TrapContext in user space, sscratch->user stack
    # save other general purpose registers
    sd x1, 1*8(sp)
    # skip sp(x2), we will save it later
    sd x3, 3*8(sp)
    # skip tp(x4), application does not use it
    # save x5~x31
    .set n, 5
    .rept 27
        SAVE_GP %n
        .set n, n+1
    .endr
    # we can use t0/t1/t2 freely, because they have been saved in TrapContext
    csrr t0, sstatus
    csrr t1, sepc
    sd t0, 32*8(sp)
    sd t1, 33*8(sp)
    # read user stack from sscratch and save it in TrapContext
    csrr t2, sscratch
    sd t2, 2*8(sp)
    # 2. 切换到内核地址空间并跳转到 trap handler
    ## 用于切换页表
    # load kernel_satp into t0
    ld t0, 34*8(sp)
    # load trap_handler into t1
    ld t1, 36*8(sp)
    # move to kernel_sp (现在将sp修改为内核栈顶地址)
    ld sp, 35*8(sp)
    # switch to kernel space (将内核页表地址写入satp, 并刷新快表)
    csrw satp, t0
    sfence.vma
    # jump to trap_handler
    jr t1

__restore:
    # a0: *TrapContext in user space(Constant); a1: user space token (某应用页表地址)
    # switch to user space
    csrw satp, a1
    sfence.vma
    # 将传入的 Trap 上下文位置保存在 sscratch 寄存器中，这样 __alltraps 中才能基于它将 Trap 上下文保存到正确的位置；
    csrw sscratch, a0
    # 将sp修改为trap上下文的地址,用于恢复寄存器
    mv sp, a0
    # now sp points to TrapContext in user space, start restoring based on it
    # restore sstatus/sepc
    ld t0, 32*8(sp)
    ld t1, 33*8(sp)
    csrw sstatus, t0
    csrw sepc, t1
    # restore general purpose registers except x0/sp/tp
    ld x1, 1*8(sp)
    ld x3, 3*8(sp)
    .set n, 5
    .rept 27
        LOAD_GP %n
        .set n, n+1
    .endr
    # back to user stack (对应x2, 就是sp指针)
    ld sp, 2*8(sp)
    sret
