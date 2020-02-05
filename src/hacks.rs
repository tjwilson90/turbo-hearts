use std::ops::{Deref, DerefMut};
use tokio::{
    stream::{Stream, StreamExt},
    sync,
    sync::mpsc,
};

// hacks to get ide recognition for these types in every other file,
// see https://github.com/intellij-rust/intellij-rust/issues/4627

pub struct Mutex<T>(sync::Mutex<T>);
pub struct MutexGuard<'a, T>(sync::MutexGuard<'a, T>);
pub struct UnboundedSender<T>(mpsc::UnboundedSender<T>);
pub struct UnboundedReceiver<T>(mpsc::UnboundedReceiver<T>);

impl<T> Mutex<T> {
    pub fn new(value: T) -> Self {
        Self(sync::Mutex::new(value))
    }

    pub async fn lock(&self) -> MutexGuard<'_, T> {
        MutexGuard(self.0.lock().await)
    }
}

impl<'a, T> Deref for MutexGuard<'a, T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &*self.0
    }
}

impl<'a, T> DerefMut for MutexGuard<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut *self.0
    }
}

pub fn unbounded_channel<T>() -> (UnboundedSender<T>, UnboundedReceiver<T>) {
    let (tx, rx) = mpsc::unbounded_channel();
    (UnboundedSender(tx), UnboundedReceiver(rx))
}

impl<T> UnboundedSender<T> {
    pub fn send(&self, message: T) -> Result<(), ()> {
        self.0.send(message).map_err(|_| ())
    }
}

impl<T> UnboundedReceiver<T> {
    pub fn map<U, F>(self, f: F) -> impl Stream<Item = U>
    where
        F: FnMut(T) -> U,
        Self: Sized,
    {
        self.0.map(f)
    }
}
