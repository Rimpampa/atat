mod client;
mod simple_client;

pub use client::Client;
pub use simple_client::SimpleClient;

use crate::{AtatCmd, CmdResult, Error};

pub trait AtatClient<Cmd: AtatCmd> {
    /// Send an AT command.
    ///
    /// `cmd` must implement [`AtatCmd`].
    ///
    /// This function will also make sure that at least `self.config.cmd_cooldown`
    /// has passed since the last response or URC has been received, to allow
    /// the slave AT device time to deliver URC's.
    async fn send(&mut self, cmd: &Cmd) -> CmdResult<Cmd>;

    async fn send_retry(&mut self, cmd: &Cmd) -> CmdResult<Cmd> {
        for attempt in 1..=Cmd::ATTEMPTS {
            if attempt > 1 {
                debug!("Attempt {}:", attempt);
            }

            match self.send(cmd).await {
                Err(Error::Timeout) => {}
                Err(Error::Parse) => {
                    if !Cmd::REATTEMPT_ON_PARSE_ERR {
                        return Err(Error::Parse);
                    }
                }
                r => return r,
            }
        }
        Err(Error::Timeout)
    }
}

impl<T, Cmd: AtatCmd> AtatClient<Cmd> for &mut T
where
    T: AtatClient<Cmd>,
{
    async fn send(&mut self, cmd: &Cmd) -> CmdResult<Cmd> {
        T::send(self, cmd).await
    }
}
