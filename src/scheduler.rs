use std::cell::RefCell;
use std::collections::VecDeque;
use std::rc::Rc;

pub type Shared<T> = Rc<RefCell<T>>;

thread_local! {
    static SCHEDULER: Rc<Scheduler> =
        Rc::new(Scheduler::new());
}

pub fn scheduler() -> Rc<Scheduler> {
    SCHEDULER.with(Rc::clone)
}

/// A routine which could be run.
pub trait Runnable {
    /// Runs a routine with a context instance.
    fn run(self: Box<Self>);
}

/// This is a global scheduler suitable to schedule and run any tasks.
#[derive(Clone)]
pub struct Scheduler {
    /// This lock is used to prevent recursion in [Scheduler#start()](Scheduler#start())
    lock: Rc<RefCell<()>>,
    main: Shared<VecDeque<Box<dyn Runnable>>>,
    component: ComponentScheduler,
}

pub enum ComponentRunnableType {
    Destroy,
    Create,
    Update,
    Render,
}

#[derive(Clone)]
struct ComponentScheduler {
    // Queues
    destroy: Shared<VecDeque<Box<dyn Runnable>>>,
    create: Shared<VecDeque<Box<dyn Runnable>>>,
    update: Shared<VecDeque<Box<dyn Runnable>>>,

    // Stack
    render: Shared<Vec<Box<dyn Runnable>>>,
}

impl ComponentScheduler {
    fn new() -> Self {
        ComponentScheduler {
            destroy: Rc::new(RefCell::new(VecDeque::new())),
            create: Rc::new(RefCell::new(VecDeque::new())),
            update: Rc::new(RefCell::new(VecDeque::new())),
            render: Rc::new(RefCell::new(Vec::new())),
        }
    }

    fn next_runnable(&self) -> Option<Box<dyn Runnable>> {
        None.or_else(|| self.destroy.borrow_mut().pop_front())
            .or_else(|| self.create.borrow_mut().pop_front())
            .or_else(|| self.update.borrow_mut().pop_front())
            .or_else(|| self.render.borrow_mut().pop())
    }
}

impl Scheduler {
    fn new() -> Self {
        Scheduler {
            lock: Rc::new(RefCell::new(())),
            main: Rc::new(RefCell::new(VecDeque::new())),
            component: ComponentScheduler::new(),
        }
    }

    pub fn push_comp(&self, run_type: ComponentRunnableType, runnable: Box<dyn Runnable>) {
        match run_type {
            ComponentRunnableType::Destroy => {
                self.component.destroy.borrow_mut().push_back(runnable)
            }
            ComponentRunnableType::Create => self.component.create.borrow_mut().push_back(runnable),
            ComponentRunnableType::Update => self.component.update.borrow_mut().push_back(runnable),
            ComponentRunnableType::Render => self.component.render.borrow_mut().push(runnable),
        };
        self.start();
    }

    pub fn push(&self, runnable: Box<dyn Runnable>) {
        self.main.borrow_mut().push_back(runnable);
        self.start();
    }

    fn next_runnable(&self) -> Option<Box<dyn Runnable>> {
        None.or_else(|| self.component.next_runnable())
            .or_else(|| self.main.borrow_mut().pop_front())
    }

    pub fn start(&self) {
        // The lock is used to prevent recursion. If the lock
        // cannot be acquired, it is because the `start()` method
        // is being called recursively as part of a `runnable.run()`.
        if let Ok(_lock) = self.lock.try_borrow_mut() {
            while let Some(runnable) = self.next_runnable() {
                runnable.run();
            }
        }
    }
}