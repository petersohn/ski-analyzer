use serde::Serialize;
use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager, Runtime};

use ski_analyzer_lib::error::{Error, ErrorType, Result};
use ski_analyzer_lib::utils::cancel::{
    Cancellable, CancellableTask, CancellationToken,
};

use crate::utils::event::emit_event;

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

    pub fn cancel_task(&mut self, task_id: u64) {
        if let Some(task) = self.active_tasks.get_mut(&task_id) {
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

#[derive(Serialize, Clone)]
pub struct TaskResult<T>
where
    T: Serialize + Clone,
{
    task_id: u64,
    data: T,
}

pub fn do_with_task<Fut, R, Ret, Err>(
    app_handle: AppHandle<R>,
    func: impl FnOnce(TaskHandle) -> Fut,
) -> u64
where
    Fut: Future<Output = std::result::Result<Ret, Err>> + Send + 'static,
    Ret: Serialize + Clone,
    Err: Serialize + Clone,
    R: Runtime,
{
    let task_manager = app_handle.state::<TaskManagerType>();
    let task = TaskManager::add_task((*task_manager).clone());
    let task_id = task.id;
    let future = func(task);

    tauri::async_runtime::spawn(async move {
        match future.await {
            Ok(data) => emit_event(
                &app_handle,
                "task_finished",
                &TaskResult { task_id, data },
            ),
            Err(data) => emit_event(
                &app_handle,
                "task_failed",
                &TaskResult { task_id, data },
            ),
        }
    });

    task_id
}
