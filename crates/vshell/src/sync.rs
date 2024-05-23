use parking_lot::{Mutex, MutexGuard};

pub struct FairMutex<T> {
    data: Mutex<T>,
    next: Mutex<()>,
}

impl<T> FairMutex<T> {
    pub fn new(data: T) -> FairMutex<T> {
        FairMutex {
            data: Mutex::new(data),
            next: Mutex::new(()),
        }
    }

    pub fn lease(&self) -> MutexGuard<'_, ()> {
        self.next.lock()
    }

    pub fn lock(&self) -> MutexGuard<'_, T> {
        let _next = self.next.lock();
        self.data.lock()
    }

    pub fn lock_unfair(&self) -> MutexGuard<'_, T> {
        self.data.lock()
    }

    pub fn try_lock_unfair(&self) -> Option<MutexGuard<'_, T>> {
        self.data.try_lock()
    }
}
