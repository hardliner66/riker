use std::sync::Arc;

use futures::{channel::mpsc::Sender, task::SpawnExt, SinkExt};

#[cfg(feature = "profiling")]
use tracing_futures::Instrument;

use crate::{
    actor::{MsgError, MsgResult},
    kernel::{
        mailbox::{AnySender, MailboxSchedule, MailboxSender},
        KernelMsg,
    },
    system::ActorSystem,
    AnyMessage, Envelope, Message,
};

#[derive(Debug, Clone)]
pub struct KernelRef {
    pub tx: Sender<KernelMsg>,
}

impl KernelRef {
    #[cfg_attr(
        all(feature = "profiling", feature = "optick-profiler"),
        optick_attr::profile
    )]
    #[cfg_attr(
        all(feature = "profiling", not(feature = "optick-profiler")),
        instrument
    )]
    pub(crate) fn schedule(&self, sys: &ActorSystem) {
        self.send(KernelMsg::RunActor, sys);
    }

    #[cfg_attr(
        all(feature = "profiling", feature = "optick-profiler"),
        optick_attr::profile
    )]
    #[cfg_attr(
        all(feature = "profiling", not(feature = "optick-profiler")),
        instrument
    )]
    pub(crate) fn restart(&self, sys: &ActorSystem) {
        self.send(KernelMsg::RestartActor, sys);
    }

    #[cfg_attr(
        all(feature = "profiling", feature = "optick-profiler"),
        optick_attr::profile
    )]
    #[cfg_attr(
        all(feature = "profiling", not(feature = "optick-profiler")),
        instrument
    )]
    pub(crate) fn terminate(&self, sys: &ActorSystem) {
        self.send(KernelMsg::TerminateActor, sys);
    }

    #[cfg_attr(
        all(feature = "profiling", feature = "optick-profiler"),
        optick_attr::profile
    )]
    #[cfg_attr(
        all(feature = "profiling", not(feature = "optick-profiler")),
        instrument
    )]
    pub(crate) fn sys_init(&self, sys: &ActorSystem) {
        self.send(KernelMsg::Sys(sys.clone()), sys);
    }

    #[cfg_attr(
        all(feature = "profiling", feature = "optick-profiler"),
        optick_attr::profile
    )]
    #[cfg_attr(
        all(feature = "profiling", not(feature = "optick-profiler")),
        instrument
    )]
    fn send(&self, msg: KernelMsg, sys: &ActorSystem) {
        let mut tx = self.tx.clone();
        sys.exec
            .spawn(internal_trace_future!(async move {
                internal_trace_span!("future");
                drop(tx.send(msg).await);
            }))
            .unwrap();
    }
}

#[cfg_attr(
    all(feature = "profiling", feature = "optick-profiler"),
    optick_attr::profile
)]
#[cfg_attr(
    all(feature = "profiling", not(feature = "optick-profiler")),
    instrument(skip(mbox))
)]
pub fn dispatch<Msg>(
    msg: Envelope<Msg>,
    mbox: &MailboxSender<Msg>,
    kernel: &KernelRef,
    sys: &ActorSystem,
) -> MsgResult<Envelope<Msg>>
where
    Msg: Message,
{
    match mbox.try_enqueue(msg) {
        Ok(_) => {
            if !mbox.is_scheduled() {
                mbox.set_scheduled(true);
                kernel.schedule(sys);
            }

            Ok(())
        }
        Err(e) => Err(MsgError::new(e.msg)),
    }
}

#[cfg_attr(
    all(feature = "profiling", feature = "optick-profiler"),
    optick_attr::profile
)]
#[cfg_attr(
    all(feature = "profiling", not(feature = "optick-profiler")),
    instrument(skip(mbox))
)]
pub fn dispatch_any(
    msg: &mut AnyMessage,
    sender: crate::actor::Sender,
    mbox: &Arc<dyn AnySender>,
    kernel: &KernelRef,
    sys: &ActorSystem,
) -> Result<(), ()> {
    match mbox.try_any_enqueue(msg, sender) {
        Ok(_) => {
            if !mbox.is_sched() {
                mbox.set_sched(true);
                kernel.schedule(sys);
            }

            Ok(())
        }
        Err(_) => Err(()),
    }
}

unsafe impl Send for KernelRef {}
unsafe impl Sync for KernelRef {}
