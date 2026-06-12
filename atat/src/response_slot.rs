use core::cell::RefCell;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    mutex::{Mutex, MutexGuard},
    signal::Signal,
};
use heapless::Vec;

use crate::{InternalError, NoCustomError, Response};

pub struct ResponseSlot<const N: usize, E = NoCustomError>(
    Mutex<CriticalSectionRawMutex, RefCell<Response<N, E>>>,
    Signal<CriticalSectionRawMutex, ()>,
);

pub type ResponseSlotGuard<'a, const N: usize, E = NoCustomError> =
    MutexGuard<'a, CriticalSectionRawMutex, RefCell<Response<N, E>>>;

#[derive(Debug)]
pub struct SlotInUseError;

impl<const N: usize, E> Default for ResponseSlot<N, E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<const N: usize, E> ResponseSlot<N, E> {
    pub const fn new() -> Self {
        Self(
            Mutex::new(RefCell::new(Response::Ok(Vec::new()))),
            Signal::new(),
        )
    }

    /// Reset the current response slot
    pub fn reset(&self) {
        self.1.reset();
    }

    /// Wait for a response to be signaled and get a guard to the response
    pub async fn get<'a>(&'a self) -> ResponseSlotGuard<'a, N, E> {
        self.1.wait().await;

        // The mutex is not locked when signal is emitted
        self.0.try_lock().unwrap()
    }

    /// If signaled, get a guard to the response
    pub fn try_get<'a>(&'a self) -> Option<ResponseSlotGuard<'a, N, E>> {
        if self.1.signaled() {
            // The mutex is not locked when signal is emitted
            Some(self.0.try_lock().unwrap())
        } else {
            None
        }
    }

    pub(crate) fn signal_prompt(&self, prompt: u8) -> Result<(), SlotInUseError> {
        if self.1.signaled() {
            return Err(SlotInUseError);
        }

        // Not currently signaled: We know that the client is not currently holding the response slot guard
        {
            let buf = self.0.try_lock().unwrap();
            let mut res = buf.borrow_mut();
            *res = Response::Prompt(prompt);
        }

        // Mutex is unlocked before we signal
        self.1.signal(());
        Ok(())
    }

    pub(crate) fn signal_response<'a>(
        &self,
        response: Result<&'a [u8], InternalError<'a>>,
    ) -> Result<(), SlotInUseError>
    where
        E: From<&'a [u8]>,
    {
        if self.1.signaled() {
            return Err(SlotInUseError);
        }

        // Not currently signaled: We know that the client is not currently holding the response slot guard
        {
            let buf = self.0.try_lock().unwrap();
            let mut res = buf.borrow_mut();
            *res = response.into();
        }

        // Mutex is unlocked before we signal
        self.1.signal(());
        Ok(())
    }
}
