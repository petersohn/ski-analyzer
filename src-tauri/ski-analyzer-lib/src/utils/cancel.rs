use crate::error::{Error, ErrorType, Result};
use futures::future::FutureExt;
use std::{
    future::Future,
    sync::atomic::{AtomicBool, Ordering},
};
use tokio::task::AbortHandle;

pub trait Cancellable {
    fn cancel(&self);
}

pub struct CancellationToken {
    cancelled: AtomicBool,
}

impl CancellationToken {
    pub fn new() -> Self {
        CancellationToken {
            cancelled: AtomicBool::new(false),
        }
    }

    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Relaxed)
    }

    pub fn check(&self) -> Result<()> {
        if self.is_cancelled() {
            Err(Error::new_s(ErrorType::Cancelled, "cancelled"))
        } else {
            Ok(())
        }
    }
}

impl Cancellable for CancellationToken {
    fn cancel(&self) {
        self.cancelled.store(true, Ordering::Relaxed);
    }
}

pub struct CancellableTask {
    abort_handle: AbortHandle,
}

impl CancellableTask {
    pub fn spawn<F, R>(
        future: F,
    ) -> (impl Future<Output = F::Output>, CancellableTask)
    where
        F: Future<Output = Result<R>> + Send + 'static,
        R: Send + 'static,
    {
        let handle = tokio::spawn(future);
        let task = CancellableTask {
            abort_handle: handle.abort_handle(),
        };
        let fut = handle.map(|res| match res {
            Ok(r) => r,
            Err(e) => {
                if e.is_cancelled() {
                    Err(Error::new_s(ErrorType::Cancelled, "cancelled"))
                } else {
                    Err(Error::new(ErrorType::ExternalError, e.to_string()))
                }
            }
        });
        (fut, task)
    }
}

impl Cancellable for CancellableTask {
    fn cancel(&self) {
        self.abort_handle.abort();
    }
}
