use core::future::{poll_fn, Future};
use core::sync::atomic::{AtomicBool, Ordering};
use core::task::{Context, Poll};

use atomic_waker::AtomicWaker;

pub struct Notification {
    waker: AtomicWaker,
    triggered: AtomicBool,
}

impl Notification {
    pub const fn new() -> Self {
        Self {
            waker: AtomicWaker::new(),
            triggered: AtomicBool::new(false),
        }
    }

    pub fn reset(&self) {
        self.triggered.store(false, Ordering::SeqCst);
        self.waker.take();
    }

    pub fn notify(&self) {
        self.triggered.store(true, Ordering::SeqCst);
        self.waker.wake();
    }

    pub fn triggered(&self) -> bool {
        self.triggered.load(Ordering::SeqCst)
    }

    pub fn wait(&self) -> impl Future<Output = ()> + '_ {
        poll_fn(move |cx| self.poll_wait(cx))
    }

    fn poll_wait(&self, cx: &mut Context<'_>) -> Poll<()> {
        self.waker.register(cx.waker());

        if self.triggered.swap(false, Ordering::SeqCst) {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }
}

impl Default for Notification {
    fn default() -> Self {
        Self::new()
    }
}
