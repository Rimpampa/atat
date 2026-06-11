mod blocking_timer;
mod client;

pub use client::Client;

use crate::{AtatCmd, CmdResult, Error};

pub trait AtatClient {
    /// Send an AT command.
    ///
    /// `cmd` must implement [`AtatCmd`].
    ///
    /// This function will block until a response is received, if in Timeout or
    /// Blocking mode. In Nonblocking mode, the send can be called until it no
    /// longer returns `nb::Error::WouldBlock`, or `self.check_response(cmd)` can
    /// be called, with the same result.
    ///
    /// This function will also make sure that at least `self.config.cmd_cooldown`
    /// has passed since the last response or URC has been received, to allow
    /// the slave AT device time to deliver URC's.
    fn send<Cmd: AtatCmd>(&mut self, cmd: &Cmd) -> CmdResult<Cmd>;

    fn send_retry<Cmd: AtatCmd>(&mut self, cmd: &Cmd) -> CmdResult<Cmd> {
        for attempt in 1..=Cmd::ATTEMPTS {
            if attempt > 1 {
                debug!("Attempt {}:", attempt);
            }

            match self.send(cmd) {
                Err(Error::Timeout) => {}
                r => return r,
            }
        }
        Err(Error::Timeout)
    }
}
