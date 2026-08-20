## 执行流程

- 启动前：`init()` 把 ``__alltraps`` 地址写入 `stvec`。
- 运行时：用户程序想写文件，设置好 `a7=64`，执行 `ecall`。
- 硬件跳转：CPU 自动保存 `pc` 到 `sepc`，跳到 `__alltraps`。
- 保存现场：`__alltraps` 把寄存器保存到内核栈的 `TrapContext`。
- 处理请求：`Rust` 代码读取 `TrapContext` 中的 `a7`，分发到对应的处理函数（如 `sys_write`）。
- 恢复现场：处理完后，修改 ``TrapContext`` 中的 ``sepc``（跳过 ``ecall`` 指令），调用 ``__restore``。
- 返回用户态：`sret` 指令根据 `sepc` 跳回用户程序，继续执行 `ecall` 后面的代码。
