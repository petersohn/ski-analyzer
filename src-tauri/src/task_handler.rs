use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};

use tauri::{Manager, Runtime};

use ski_analyzer_lib::error::Result;
use ski_analyzer_lib::utils::cancel::{
    Cancellable, CancellableTask, CancellationToken,
};

pub type TaskHandlerType = Arc<Mutex<TaskHandler>>;

#[derive(Default)]
pub struct TaskHandler {
    task_id: u64,
    active_tasks: HashMap<u64, Arc<dyn Cancellable + Send + Sync>>,
}

impl TaskHandler {
    fn add_task(&mut self, cancel: Arc<dyn Cancellable + Send + Sync>) -> u64 {
        self.task_id += 1;
        self.active_tasks.insert(self.task_id, cancel);
        self.task_id
    }

    fn remove_task(&mut self, id: u64) {
        self.active_tasks.remove(&id);
    }

    pub fn cancel_all_tasks(&mut self) {
        for (_, task) in &self.active_tasks {
            task.cancel();
        }
        self.active_tasks.clear();
    }

    pub fn add_sync_task<M, R, F, Ret>(manager: &M, func: F) -> Result<Ret>
    where
        M: Manager<R>,
        R: Runtime,
        F: FnOnce(&CancellationToken) -> Result<Ret>,
    {
        let state = manager.state::<TaskHandlerType>();
        let cancel = Arc::new(CancellationToken::new());
        let task_id = state.lock().unwrap().add_task(cancel.clone());
        let ret = func(&*cancel);
        state.lock().unwrap().remove_task(task_id);
        ret
    }

    pub async fn add_async_task<M, R, F, Ret>(
        manager: &M,
        future: F,
    ) -> Result<Ret>
    where
        M: Manager<R>,
        R: Runtime,
        F: Future<Output = Result<Ret>> + Send + 'static,
        Ret: Send + 'static,
    {
        let state = manager.state::<TaskHandlerType>();
        let (fut, cancel) = CancellableTask::spawn(future);
        let task_id = state.lock().unwrap().add_task(Arc::new(cancel));
        let ret = fut.await;
        state.lock().unwrap().remove_task(task_id);
        ret
    }
}
