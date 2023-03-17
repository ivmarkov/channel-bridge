#[cfg(target_has_atomic = "ptr")]
pub use atomic_notification::*;

#[cfg(not(target_has_atomic = "ptr"))]
pub use signal_notification::*;

impl Default for Notification {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(target_has_atomic = "ptr")]
mod atomic_notification {
    use core::future::{poll_fn, Future};
    use core::sync::atomic::{AtomicBool, Ordering};
    use core::task::{Context, Poll};

    use futures_util::task::AtomicWaker;

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

        pub fn poll_wait(&self, cx: &mut Context<'_>) -> Poll<()> {
            self.waker.register(cx.waker());

            if self.triggered.swap(false, Ordering::SeqCst) {
                Poll::Ready(())
            } else {
                Poll::Pending
            }
        }
    }
}

#[cfg(not(target_has_atomic = "ptr"))]
mod signal_notification {
    use core::future::Future;

    use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
    use embassy_sync::signal::Signal;

    pub struct Notification(Signal<CriticalSectionRawMutex, ()>);

    impl Notification {
        pub const fn new() -> Self {
            Self(Signal::new())
        }

        pub fn reset(&self) {
            self.0.reset();
        }

        pub fn notify(&self) {
            self.0.signal(());
        }

        pub fn triggered(&self) -> bool {
            self.0.signaled()
        }

        pub fn wait(&self) -> impl Future<Output = ()> + '_ {
            self.0.wait()
        }
    }
}
