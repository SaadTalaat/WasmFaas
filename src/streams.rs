//use tokio::sync::mpsc::{channel, Receiver, Sender};
use crossbeam_channel::{
    unbounded as channel, Receiver, RecvError, SendError, Sender, TryRecvError,
};

#[derive(Debug)]
pub struct Stream<In, Out>
where
    In: std::fmt::Debug,
    Out: std::fmt::Debug,
{
    sender: Sender<In>,
    receiver: Receiver<Out>,
}

impl<In, Out> Stream<In, Out>
where
    In: std::fmt::Debug,
    Out: std::fmt::Debug,
{
    pub fn new(sender: Sender<In>, receiver: Receiver<Out>) -> Self {
        Self { sender, receiver }
    }

    pub fn split(self) -> (Sender<In>, Receiver<Out>) {
        (self.sender, self.receiver)
    }

    pub fn send(&self, msg: In) -> Result<(), SendError<In>> {
        self.sender.send(msg)
    }

    pub fn recv(&self) -> Result<Out, RecvError> {
        self.receiver.recv()
    }

    pub fn try_recv(&self) -> Result<Out, TryRecvError> {
        self.receiver.try_recv()
    }
}

/// Duplex stream has two streams
/// 1. Sends T and receives U
/// 2. Other sends U and receives T
///
/// Example:
/// Given T, U are protocol types.
/// Given X, Y are two communicating parties.
/// T -> is a protocol that X sends to Y
/// U -> is a protocol that Y sends to X
/// Then
/// X needs a stream that sends T to Y and receives U from Y
/// Y needs a stream that sends U to X and receives T from X
pub fn duplex<T, U>() -> (Stream<T, U>, Stream<U, T>)
where
    T: std::fmt::Debug,
    U: std::fmt::Debug,
{
    let (t_sender, t_receiver) = channel::<T>();
    let (u_sender, u_receiver) = channel::<U>();
    (
        Stream::<T, U>::new(t_sender, u_receiver),
        Stream::<U, T>::new(u_sender, t_receiver),
    )
}
