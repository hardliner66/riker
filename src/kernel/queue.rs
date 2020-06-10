use std::{
    sync::{
        mpsc::{channel, sync_channel, Receiver, Sender, SyncSender /* , SendError */},
        Mutex,
    }
};

use crate::{Envelope, Message};

#[derive(Clone)]
enum Channel<T: Message> {
    Bounded(SyncSender<T>),
    Unbounded(Sender<T>),
    Flume(flume::Sender<T>),
}

impl<T: Message> Channel<T> {
    fn send(&self, t: T) -> Result<(), EnqueueError<T>> {
        match self {
            Channel::Bounded(tx) => tx.send(t).map_err(|e| EnqueueError { msg: e.0 }),
            Channel::Unbounded(tx) => tx.send(t).map_err(|e| EnqueueError { msg: e.0 }),
            Channel::Flume(tx) => tx.send(t).map_err(|e| EnqueueError { msg: e.0 }),
        }
    }
}

enum ChannelRecv<T: Message> {
    Std(Receiver<T>),
    Flume(flume::Receiver<T>),
}

#[derive(Debug)]
enum RecvError {
    Std(std::sync::mpsc::RecvError),
    Flume(flume::RecvError),
}

#[derive(Debug)]
enum TryRecvError {
    Std(std::sync::mpsc::TryRecvError),
    Flume(flume::TryRecvError),
}

impl<T: Message> ChannelRecv<T> {
    fn recv(&self) -> Result<T, RecvError> {
        match self {
            ChannelRecv::Std(r) => r.recv().map_err(|e| RecvError::Std(e)),
            ChannelRecv::Flume(r) => r.recv().map_err(|e| RecvError::Flume(e)),
        }
    }
    fn try_recv(&self) -> Result<T, TryRecvError> {
        match self {
            ChannelRecv::Std(r) => r.try_recv().map_err(|e| TryRecvError::Std(e)),
            ChannelRecv::Flume(r) => r.try_recv().map_err(|e| TryRecvError::Flume(e)),
        }
    }
}

pub fn queue<Msg: Message>(bound: usize) -> (QueueWriter<Msg>, QueueReader<Msg>) {

    let use_flume = false;

    let (tx, rx) = if use_flume {
        if bound > 0 {
            let (tx, rx) = flume::bounded::<Envelope<Msg>>(bound);

            (Channel::Flume(tx), ChannelRecv::Flume(rx))
        } else {
            let (tx, rx) = flume::unbounded::<Envelope<Msg>>();
            (Channel::Flume(tx), ChannelRecv::Flume(rx))
        }
    } else {
        if bound > 0 {
            let (tx, rx) = sync_channel::<Envelope<Msg>>(bound);

            (Channel::Bounded(tx), ChannelRecv::Std(rx))
        } else {
            let (tx, rx) = channel::<Envelope<Msg>>();
            (Channel::Unbounded(tx), ChannelRecv::Std(rx))
        }
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

#[derive(Clone)]
pub struct QueueWriter<Msg: Message> {
    tx: Channel<Envelope<Msg>>,
}

impl<Msg: Message> QueueWriter<Msg> {
    pub fn try_enqueue(&self, msg: Envelope<Msg>) -> EnqueueResult<Msg> {
        self.tx
            .send(msg)
            .map(|_| ())
            // .map_err(|e| EnqueueError { msg: e.0 })
    }
}

pub struct QueueReader<Msg: Message> {
    inner: Mutex<QueueReaderInner<Msg>>,
}

struct QueueReaderInner<Msg: Message> {
    rx: ChannelRecv<Envelope<Msg>>,
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
