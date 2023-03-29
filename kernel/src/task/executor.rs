use super::{Task, TaskId};
use alloc::{collections::BTreeMap, sync::Arc, task::Wake};
use core::future::Future;
use core::task::{Context, Poll, Waker};
use crossbeam_queue::ArrayQueue;

const MAX_TASKS: usize = 100;

#[derive(Clone)]
#[repr(transparent)]
pub struct Spawner(Arc<ArrayQueue<Task>>);
impl Spawner {
    pub fn new(capacity: usize) -> Self {
        Self(Arc::new(ArrayQueue::new(capacity)))
    }
    pub fn add(&self, future: impl Future<Output = ()> + 'static) {
        let _ = self.0.push(Task::new(future));
    }
}

pub struct Executor {
    tasks: BTreeMap<TaskId, Task>,
    task_queue: Arc<ArrayQueue<TaskId>>,
    spawner: Spawner,
    waker_cache: BTreeMap<TaskId, Waker>,
}

impl Executor {
    pub fn new(spawner: Spawner) -> Self {
        Self {
            tasks: BTreeMap::new(),
            task_queue: Arc::new(ArrayQueue::new(MAX_TASKS)),
            spawner,
            waker_cache: BTreeMap::new(),
        }
    }
    fn spawn(&mut self, task: Task) {
        let task_id = task.id;
        if self.tasks.insert(task.id, task).is_some() {
            panic!("Task with same ID already in task queue!!");
        }
        self.task_queue.push(task_id).expect("Task queue full!!");
    }
    fn sleep_if_idle(&self) {
        use x86_64::instructions::interrupts::{self, enable_and_hlt};
        interrupts::disable();
        if self.task_queue.is_empty() {
            enable_and_hlt();
        } else {
            interrupts::enable();
        }
    }
    pub fn run(&mut self) -> ! {
        while let Some(e) = self.spawner.0.pop() {
            self.spawn(e);
        }
        loop {
            self.run_ready_tasks();
            self.sleep_if_idle();
        }
    }

    fn run_ready_tasks(&mut self) {
        let Self {
            tasks,
            task_queue,
            waker_cache,
            ..
        } = self;

        while let Some(task_id) = task_queue.pop() {
            let task = match tasks.get_mut(&task_id) {
                Some(task) => task,
                None => continue, // task no longer exists
            };
            let waker = waker_cache
                .entry(task_id)
                .or_insert_with(|| TaskWaker::new(task_id, task_queue.clone()));
            let mut context = Context::from_waker(waker);
            match task.poll(&mut context) {
                Poll::Ready(()) => {
                    // task done -> remove it
                    tasks.remove(&task_id);
                    waker_cache.remove(&task_id);
                }
                Poll::Pending => {}
            }
        }
    }
}

struct TaskWaker {
    task_id: TaskId,
    task_queue: Arc<ArrayQueue<TaskId>>,
}
impl Wake for TaskWaker {
    fn wake(self: Arc<Self>) {
        self.wake_task();
    }
    fn wake_by_ref(self: &Arc<Self>) {
        self.wake_task();
    }
}
impl TaskWaker {
    fn wake_task(&self) {
        self.task_queue.push(self.task_id).expect("task_queue full");
    }
    #[allow(clippy::new_ret_no_self)]
    fn new(task_id: TaskId, task_queue: Arc<ArrayQueue<TaskId>>) -> Waker {
        Waker::from(Arc::new(TaskWaker {
            task_id,
            task_queue,
        }))
    }
}
