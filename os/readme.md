把任务抽象进化成了进程抽象，其主要改动集中在进程管理的功能上，即通过提供新的系统调用服务：sys_fork(创建子进程)、sys_waitpid(等待子进程结束并回收子进程资源)、sys_exec（用新的应用内容覆盖当前进程，即达到执行新应用的目的）。

为了让用户能够输入命令或执行程序的名字，ProcessOS还增加了一个 read 系统调用服务，这样用户通过操作系统的命令行接口 – 新添加的 shell 应用程序发出命令，来动态地执行各种新的应用，提高了用户与操作系统之间的交互能力。

> 打一个比方，可执行文件本身可以看成一张编译器解析源代码之后总结出的一张记载如何利用各种硬件资源进行一轮生产流程的 蓝图 。而内核的一大功能便是作为一个硬件资源管理器，它可以随时启动一轮生产流程（即执行任意一个应用），这需要选中一张蓝图（此时确定执行哪个可执行文件），接下来就需要内核按照蓝图上所记载的对资源的需求来对应的将各类资源分配给它，让这轮生产流程得以顺利进行。当按照蓝图上的记载生产流程完成（应用退出）之后，内核还需要将对应的硬件资源回收以便后续的重复利用。

## 变动

- 初始进程的创建：在内核初始化的时候需要调用 os/src/task/mod.rs 中的 add_initproc 函数，它会调用 TaskControlBlock::new 读取并解析初始应用 initproc 的 ELF 文件数据并创建初始进程 INITPROC ，随后会将它加入到全局任务管理器 TASK_MANAGER 中参与调度。
- 进程切换机制：当一个进程退出或者是主动/被动交出 CPU 使用权之后，需要由内核将 CPU 使用权交给其他进程。在本章中我们沿用 os/src/task/mod.rs 中的 suspend_current_and_run_next 和 exit_current_and_run_next 两个接口来实现进程切换功能，但是需要适当调整它们的实现。我们需要调用 os/src/task/task.rs 中的 schedule 函数进行进程切换，它会首先切换到处理器的 idle 控制流（即 os/src/task/processor 的 Processor::run 方法），然后在里面选取要切换到的进程并切换过去。
- 进程调度机制：在进程切换的时候我们需要选取一个进程切换过去。选取进程逻辑可以参考 os/src/task/manager.rs 中的 TaskManager::fetch_task 方法。
- 进程生成机制：这主要是指 fork/exec 两个系统调用。它们的实现分别可以在 os/src/syscall/process.rs 中找到，分别基于 os/src/process/task.rs 中的 TaskControlBlock::fork/exec 。
- 进程资源回收机制：当一个进程主动退出或出错退出的时候，在 exit_current_and_run_next 中会立即回收一部分资源并在进程控制块中保存退出码；而需要等到它的父进程通过 waitpid 系统调用（与 fork/exec 两个系统调用放在相同位置）捕获到它的退出码之后，它的进程控制块才会被回收，从而该进程的所有资源都被回收。
- 进程的 I/O 输入机制：为了支持用户终端 user_shell 读取用户键盘输入的功能，还需要实现 read 系统调用，它可以在 os/src/syscall/fs.rs 中找到。

## 进程

> 创建 销毁 等待 信息 其他

## shell

## idle

[idle rCore Comment](https://rcore-os.cn/rCore-Tutorial-Book-v3/chapter5/2core-data-structures.html) 说明了为什么需要`idle_task_cx_ptr`;
