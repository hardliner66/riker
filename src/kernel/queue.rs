use std::sync::Mutex;

#[cfg(not(feature = "use_flume"))]
use std::sync::mpsc::{channel, sync_channel, Receiver, Sender, SyncSender, SendError };

#[cfg(feature = "use_flume")]
use flume::{Sender, Receiver};

use crate::{Envelope, Message};

#[derive(Clone)]
#[cfg(not(feature = "use_flume"))]
enum Channel<T: Message> {
    Bounded(SyncSender<T>),
    Unbounded(Sender<T>),
}

#[cfg(not(feature = "use_flume"))]
impl<T: Message> Channel<T> {
    fn send(&self, t: T) -> Result<(), SendError<T>> {
        match self {
            Channel::Bounded(tx) => tx.send(t),
            Channel::Unbounded(tx) => tx.send(t),
        }
    }
}

#[cfg(feature = "use_flume")]
pub fn queue<Msg: Message>(bound: usize) -> (QueueWriter<Msg>, QueueReader<Msg>) {
    let (tx, rx) = if bound > 0 {
        let (tx, rx) = flume::bounded::<Envelope<Msg>>(bound);

        (tx, rx)
    } else {
        let (tx, rx) = flume::unbounded::<Envelope<Msg>>();
        (tx, rx)
    };

    let qw = QueueWriter { tx };

    let qr = QueueReaderInner {
        rx,
        next_item: None,
    };

    let qr = QueueReader {
        inner: Mutex::new(qr),
    };

    (qw, qr)
}

#[cfg(not(feature = "use_flume"))]
pub fn queue<Msg: Message>(bound: usize) -> (QueueWriter<Msg>, QueueReader<Msg>) {
    let (tx, rx) = if bound > 0 {
        let (tx, rx) = sync_channel::<Envelope<Msg>>(bound);

        (Channel::Bounded(tx), rx)
    } else {
        let (tx, rx) = channel::<Envelope<Msg>>();
        (Channel::Unbounded(tx), rx)
    };

    let qw = QueueWriter { tx };

    let qr = QueueReaderInner {
        rx,
        next_item: None,
    };

    let qr = QueueReader {
        inner: Mutex::new(qr),
    };

    (qw, qr)
}

#[cfg(not(feature = "use_flume"))]
#[derive(Clone)]
pub struct QueueWriter<Msg: Message> {
    tx: Channel<Envelope<Msg>>,
}

#[cfg(feature = "use_flume")]
#[derive(Clone)]
pub struct QueueWriter<Msg: Message> {
    tx: Sender<Envelope<Msg>>,
}

impl<Msg: Message> QueueWriter<Msg> {
    pub fn try_enqueue(&self, msg: Envelope<Msg>) -> EnqueueResult<Msg> {
        self.tx
            .send(msg)
            .map(|_| ())
            .map_err(|e| EnqueueError { msg: e.0 })
    }
}

pub struct QueueReader<Msg: Message> {
    inner: Mutex<QueueReaderInner<Msg>>,
}

struct QueueReaderInner<Msg: Message> {
    rx: Receiver<Envelope<Msg>>,
    next_item: Option<Envelope<Msg>>,
}

impl<Msg: Message> QueueReader<Msg> {
    #[allow(dead_code)]
    pub fn dequeue(&self) -> Envelope<Msg> {
        let mut inner = self.inner.lock().unwrap();
        if let Some(item) = inner.next_item.take() {
            item
        } else {
            inner.rx.recv().unwrap()
        }
    }

    pub fn try_dequeue(&self) -> DequeueResult<Envelope<Msg>> {
        let mut inner = self.inner.lock().unwrap();
        if let Some(item) = inner.next_item.take() {
            Ok(item)
        } else {
            inner.rx.try_recv().map_err(|_| QueueEmpty)
        }
    }

    pub fn has_msgs(&self) -> bool {
        let mut inner = self.inner.lock().unwrap();
        inner.next_item.is_some() || {
            match inner.rx.try_recv() {
                Ok(item) => {
                    inner.next_item = Some(item);
                    true
                }
                Err(_) => false,
            }
        }
    }
}

#[derive(Clone, Debug)]
pub struct EnqueueError<T> {
    pub msg: T,
}

pub type EnqueueResult<Msg> = Result<(), EnqueueError<Envelope<Msg>>>;

pub struct QueueEmpty;
pub type DequeueResult<Msg> = Result<Msg, QueueEmpty>;
