use derivative::Derivative;
use std::{error, fmt};

#[cfg(test)]
use proptest_derive::Arbitrary;

/// An error that may be returned by [`RingSender::send`].
///
/// [`RingSender::send`]: struct.RingSender.html#method.send
#[derive(Derivative)]
#[derivative(Debug)]
#[derive(Copy, Clone, Eq, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum SendError<T> {
    /// The channel is disconnected.
    Disconnected(#[derivative(Debug = "ignore")] T),
}

impl<T> fmt::Display for SendError<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        "sending on a disconnected channel".fmt(f)
    }
}

impl<T: Send> error::Error for SendError<T> {}

/// An error that may be returned by [`RingReceiver::recv`].
///
/// [`RingReceiver::recv`]: struct.RingReceiver.html#method.recv
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum RecvError {
    /// No messages pending in the internal buffer and the channel is disconnected.
    Disconnected,
}

impl fmt::Display for RecvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use RecvError::*;
        match self {
            Disconnected => "receiving on an empty and disconnected channel".fmt(f),
        }
    }
}

impl error::Error for RecvError {}

/// An error that may be returned by [`RingReceiver::try_recv`].
///
/// [`RingReceiver::try_recv`]: struct.RingReceiver.html#method.try_recv
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(test, derive(Arbitrary))]
pub enum TryRecvError {
    /// No messages pending in the internal buffer.
    Empty,

    /// No messages pending in the internal buffer and the channel is disconnected.
    Disconnected,
}

impl fmt::Display for TryRecvError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TryRecvError::*;
        match self {
            Empty => "receiving on an empty channel".fmt(f),
            Disconnected => "receiving on an empty and disconnected channel".fmt(f),
        }
    }
}

impl error::Error for TryRecvError {}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn send_error_implements_error_trait(err: SendError<()>) {
            assert_eq!(
                format!("{}", err),
                format!("{}", Box::<dyn error::Error>::from(err))
            );
        }

        #[test]
        fn recv_error_implements_error_trait(err: RecvError) {
            assert_eq!(
                format!("{}", err),
                format!("{}", Box::<dyn error::Error>::from(err))
            );
        }

        #[test]
        fn try_recv_error_implements_error_trait(err: TryRecvError) {
            assert_eq!(
                format!("{}", err),
                format!("{}", Box::<dyn error::Error>::from(err))
            );
        }
    }
}
