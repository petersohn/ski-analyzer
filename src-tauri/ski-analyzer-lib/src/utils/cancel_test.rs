use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use super::cancel::{Cancellable, CancellableTask, CancellationToken};
use crate::error::ErrorType;

#[test]
fn cancellation_token() {
    let token = CancellationToken::new();
    assert_eq!(token.is_cancelled(), false);
    assert!(token.check().is_ok());
    token.cancel();
    assert_eq!(token.is_cancelled(), true);
    match token.check() {
        Ok(_) => panic!("Should be cancelled"),
        Err(err) => assert_eq!(err.get_type(), ErrorType::Cancelled),
    };
}

#[tokio::test]
async fn cancellable_task_ok() {
    let finished = Arc::new(AtomicBool::new(false));
    let finished2 = Arc::clone(&finished);
    let (fut, _cancel) = CancellableTask::spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        finished2.store(true, Ordering::SeqCst);
        Ok(())
    });
    tokio::time::pause();
    assert!(fut.await.is_ok());
    assert!(finished.load(Ordering::SeqCst));
}

#[tokio::test]
async fn cancellable_task_cancel() {
    let finished = Arc::new(AtomicBool::new(false));
    let finished2 = Arc::clone(&finished);
    let (fut, cancel) = CancellableTask::spawn(async move {
        tokio::time::sleep(Duration::from_secs(1)).await;
        finished2.store(true, Ordering::SeqCst);
        Ok(())
    });
    cancel.cancel();
    tokio::time::pause();
    match fut.await {
        Ok(_) => panic!("Should be cancelled"),
        Err(err) => assert_eq!(err.get_type(), ErrorType::Cancelled),
    };
    assert!(!finished.load(Ordering::SeqCst));
}
