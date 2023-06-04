use std::sync::mpsc;

pub struct Channel<T> {
    sender: mpsc::Sender<T>,
    receiver: mpsc::Receiver<T>,
}

impl<T: std::cmp::PartialEq> Channel<T> {
    pub fn new_pair() -> (Channel<T>, Channel<T>) {
        let (sender1, receiver1) = mpsc::channel::<T>();
        let (sender2, receiver2) = mpsc::channel::<T>();

        let com1 = Channel {
            sender: sender1,
            receiver: receiver2,
        };
        let com2 = Channel {
            sender: sender2,
            receiver: receiver1,
        };
        (com1, com2)
    }

    pub fn wait_for(&self, waited_message: T) {
        loop {
            let message = self.receiver.recv().unwrap();
            if message == waited_message {
                break;
            }
        }
    }
    pub fn wait_for_or_timeout(
        &self,
        waited_message: T,
        timeout: std::time::Duration,
    ) -> Result<(), crate::error::ChannelError<T>> {
        let start_time = std::time::Instant::now();

        let internal_timeout = timeout / 100;
        while start_time.elapsed() < timeout {
            // we map the internal_timeout to be very small to be able to quit as soon as the timeout is done
            // + having a dynamic internal_timeout is adding to the consistency
            match self.recv_timeout(internal_timeout) {
                Ok(message) => {
                    if message == waited_message {
                        return Ok(());
                    }
                }
                Err(err) => match err {
                    crate::error::ChannelError::RecvTimeout(_mpsc_timeout_error) => {
                        // warn!("mpsc_timeout_error: {mpsc_timeout_error}")
                    }
                    _ => return Err(err),
                },
            }
        }
        Err(mpsc::RecvTimeoutError::Timeout.into())
    }
    pub fn send(&self, t: T) -> Result<(), crate::error::ChannelError<T>> {
        Ok(self.sender.send(t)?)
    }
    pub fn iter(&self) -> mpsc::Iter<'_, T> {
        self.receiver.iter()
    }
    pub fn try_iter(&self) -> mpsc::TryIter<'_, T> {
        self.receiver.try_iter()
    }
    pub fn recv(&self) -> Result<T, crate::error::ChannelError<T>> {
        Ok(self.receiver.recv()?)
    }
    pub fn try_recv(&self) -> Result<T, crate::error::ChannelError<T>> {
        Ok(self.receiver.try_recv()?)
    }
    pub fn recv_timeout(
        &self,
        timeout: std::time::Duration,
    ) -> Result<T, crate::error::ChannelError<T>> {
        Ok(self.receiver.recv_timeout(timeout)?)
    }
}
