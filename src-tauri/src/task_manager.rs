use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};

use ski_analyzer_lib::error::{Error, ErrorType, Result};
use ski_analyzer_lib::utils::cancel::{
    Cancellable, CancellableTask, CancellationToken,
};

pub type TaskManagerType = Arc<Mutex<TaskManager>>;

enum TaskState {
    Passive,
    Active(Arc<dyn Cancellable + Send + Sync>),
    Cancelled,
}

impl TaskState {
    fn activate(
        &mut self,
        task: Arc<dyn Cancellable + Send + Sync>,
    ) -> Result<()> {
        match self {
            TaskState::Passive => {
                *self = TaskState::Active(task);
                Ok(())
            }
            TaskState::Active(_) => {
                panic!("Cannot have multiple tasks active at a time.")
            }
            TaskState::Cancelled => {
                Err(Error::new_s(ErrorType::Cancelled, "cancelled"))
            }
        }
    }

    fn deactivate(&mut self) {
        if let TaskState::Active(_) = self {
            *self = TaskState::Passive;
        }
    }

    fn cancel(&mut self) {
        if let TaskState::Active(task) = self {
            task.cancel();
        }
        *self = TaskState::Cancelled;
    }
}

#[derive(Default)]
pub struct TaskManager {
    task_id: u64,
    active_tasks: HashMap<u64, TaskState>,
}

impl TaskManager {
    pub fn add_task(manager: TaskManagerType) -> TaskHandle {
        let id = {
            let mut lock = manager.lock().unwrap();
            lock.task_id += 1;
            let id = lock.task_id;
            lock.active_tasks.insert(id, TaskState::Passive);
            id
        };
        TaskHandle { manager, id }
    }

    pub fn cancel_all_tasks(&mut self) {
        for (_, task) in &mut self.active_tasks {
            task.cancel();
        }
    }

    fn get_task(&mut self, id: u64) -> &mut TaskState {
        self.active_tasks.get_mut(&id).unwrap()
    }
}

pub struct TaskHandle {
    manager: TaskManagerType,
    id: u64,
}

impl TaskHandle {
    pub fn add_sync_task<F, Ret>(&self, func: F) -> Result<Ret>
    where
        F: FnOnce(&CancellationToken) -> Result<Ret>,
    {
        let cancel = Arc::new(CancellationToken::new());
        self.manager
            .lock()
            .unwrap()
            .get_task(self.id)
            .activate(cancel.clone())?;
        let ret = func(&*cancel);
        self.manager.lock().unwrap().get_task(self.id).deactivate();
        ret
    }

    pub async fn add_async_task<F, Ret>(&self, future: F) -> Result<Ret>
    where
        F: Future<Output = Result<Ret>> + Send + 'static,
        Ret: Send + 'static,
    {
        let (fut, cancel) = CancellableTask::spawn(future);
        self.manager
            .lock()
            .unwrap()
            .get_task(self.id)
            .activate(Arc::new(cancel))?;
        let ret = fut.await;
        self.manager.lock().unwrap().get_task(self.id).deactivate();
        ret
    }
}

impl Drop for TaskHandle {
    fn drop(&mut self) {
        self.manager.lock().unwrap().active_tasks.remove(&self.id);
    }
}
