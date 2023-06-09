#[derive(thiserror::Error, Debug)]
pub enum Error<T> {
    Channel(#[from] ChannelError<T>),
}

#[derive(thiserror::Error, Debug)]
pub enum ChannelError<T> {
    #[error(transparent)]
    Send(#[from] std::sync::mpsc::SendError<T>),
    #[error(transparent)]
    TrySend(#[from] std::sync::mpsc::TrySendError<T>),
    #[error(transparent)]
    Recv(#[from] std::sync::mpsc::RecvError),
    #[error(transparent)]
    TryRecv(#[from] std::sync::mpsc::TryRecvError),
    #[error(transparent)]
    RecvTimeout(#[from] std::sync::mpsc::RecvTimeoutError),
    #[error("Encountered an error while using the channel: {0}")]
    Other(&'static str),
}
