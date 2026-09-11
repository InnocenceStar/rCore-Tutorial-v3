// https://blog.aloni.org/posts/a-stack-less-rust-coroutine-100-loc/
// https://github.com/chyyuu/example-coroutine-and-thread/tree/stackless-coroutine-x86
#![no_std]
#![no_main]
/// Rust 标准库只提供抽象（Future、Waker 等），不提供调度器。博客中的 Executor 就是作者自己实现的简易调度器。
use core::future::Future;
use core::pin::Pin;
use core::task::{Context, Poll};
use core::task::{RawWaker, RawWakerVTable, Waker};

extern crate alloc;
use alloc::collections::VecDeque;

use alloc::boxed::Box;

#[macro_use]
extern crate user_lib;

enum State {
    Halted,
    Running,
}

struct Task {
    state: State,
}

impl Task {
    fn waiter<'a>(&'a mut self) -> Waiter<'a> {
        Waiter { task: self }
    }
}

struct Waiter<'a> {
    task: &'a mut Task,
}

impl<'a> Future for Waiter<'a> {
    type Output = ();

    /// Future trait：定义了 poll() 方法，返回 Poll::Ready(T) 或 Poll::Pending，是所有异步操作的核心抽象接口。
    /// Ready(T) 表示完成，Pending 表示未完成，是 poll() 的返回值类型。
    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context) -> Poll<Self::Output> {
        match self.task.state {
            State::Halted => {
                self.task.state = State::Running;
                Poll::Ready(())
            }
            State::Running => {
                self.task.state = State::Halted;
                Poll::Pending
            }
        }
    }
}

struct Executor {
    tasks: VecDeque<Pin<Box<dyn Future<Output = ()>>>>,
}

impl Executor {
    fn new() -> Self {
        Executor {
            tasks: VecDeque::new(),
        }
    }

    fn push<C, F>(&mut self, closure: C)
    where
        F: Future<Output = ()> + 'static,
        C: FnOnce(Task) -> F,
    {
        let task = Task {
            state: State::Running,
        };
        self.tasks.push_back(Box::pin(closure(task)));
    }

    fn run(&mut self) {
        let waker = create_waker();
        let mut context = Context::from_waker(&waker);

        while let Some(mut task) = self.tasks.pop_front() {
            match task.as_mut().poll(&mut context) {
                Poll::Pending => {
                    self.tasks.push_back(task);
                }
                Poll::Ready(()) => {}
            }
        }
    }
}

pub fn create_waker() -> Waker {
    // Safety: The waker points to a vtable with functions that do nothing. Doing
    // nothing is memory-safe.
    unsafe { Waker::from_raw(RAW_WAKER) }
}

const RAW_WAKER: RawWaker = RawWaker::new(core::ptr::null(), &VTABLE);
const VTABLE: RawWakerVTable = RawWakerVTable::new(clone, wake, wake_by_ref, drop);

unsafe fn clone(_: *const ()) -> RawWaker {
    RAW_WAKER
}
unsafe fn wake(_: *const ()) {}
unsafe fn wake_by_ref(_: *const ()) {}
unsafe fn drop(_: *const ()) {}

#[unsafe(no_mangle)]
pub fn main() -> i32 {
    println!("stackless coroutine Begin..");
    let mut exec = Executor::new();
    println!(" Create futures");
    for instance in 1..=3 {
        // Rust 编译器会将 async 块自动转换为一个实现了 Future trait 的匿名状态机结构体，每个 await 点就是一个状态切割点。
        // async 的惰性求值：async 块不会立即执行，而是返回一个 Future 对象，只有被 poll() 时才推进。这是 Rust 区别于 JS/Go 的"拉模型"（pull-based）设计。
        // async → 状态机转换：编译器在编译期将 async 块转换为枚举状态机，每个 await 点对应一个状态变体，局部变量保存在结构体字段中。这是"无栈协程"（Stackless Coroutine）的核心。
        // 零成本抽象：整个转换在编译期完成，运行时没有额外的堆分配或动态分发开销，生成的机器码性能等同于手写状态机。
        exec.push(move |mut task| async move {
            println!("   Task {}: begin state", instance);
            task.waiter().await;
            println!("   Task {}: next state", instance);
            task.waiter().await;
            println!("   Task {}: end state", instance);
        });
    }

    println!(" Running");
    exec.run();
    println!(" Done");
    println!("stackless coroutine PASSED");

    0
}
