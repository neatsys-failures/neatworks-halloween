use derive_more::From;

#[derive(Debug, From)]
pub struct EventSource<M>(tokio::sync::mpsc::UnboundedReceiver<M>);

impl<M> EventSource<M> {
    pub async fn next(&mut self) -> crate::Result<M> {
        self.0
            .recv()
            .await
            .ok_or(crate::err!("unexpected source closing"))
    }

    pub async fn option_next(&mut self) -> Option<M> {
        self.0.recv().await
    }
}

#[derive(Debug, From)]
pub struct EventSender<M>(tokio::sync::mpsc::UnboundedSender<M>);

impl<M> Clone for EventSender<M> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<M> EventSender<M> {
    pub fn send(&self, message: M) -> crate::Result<()> {
        self.0
            .send(message)
            .map_err(|_| crate::err!("unexpected event channel closing"))
    }
}

/// A thin wrapper around Tokio's unbounded MPSC channel.
///
/// Wrapped to integrate with `crate::Result`.
pub fn event_channel<M>() -> (EventSender<M>, EventSource<M>) {
    let channel = tokio::sync::mpsc::unbounded_channel();
    (EventSender(channel.0), EventSource(channel.1))
}

#[derive(Debug, From)]
pub struct PromiseSender<T>(tokio::sync::oneshot::Sender<T>);

pub type PromiseSource<T> = tokio::sync::oneshot::Receiver<T>;

pub fn promise_channel<T>() -> (PromiseSender<T>, PromiseSource<T>) {
    let chan = tokio::sync::oneshot::channel();
    (PromiseSender(chan.0), chan.1)
}

impl<T> PromiseSender<T> {
    pub fn resolve(self, value: T) {
        if self.0.send(value).is_err() {
            eprintln!("return channel closed before submitted task finishes")
        }
    }
}

pub type SubmitHandle<T, U> = EventSender<(T, PromiseSender<U>)>;

impl<T, U> SubmitHandle<T, U> {
    pub async fn submit(&self, op: T) -> crate::Result<U> {
        let (result, promise) = promise_channel();
        self.send((op, result))?;
        Ok(promise.await?)
    }
}

pub type SubmitSource<T, U> = EventSource<(T, PromiseSender<U>)>;

pub type SubscribeHandle<T, U> = EventSender<(T, EventSender<U>)>;

impl<T, U> SubscribeHandle<T, U> {
    pub fn subscribe(&self, op: T) -> crate::Result<EventSource<U>> {
        let (event, source) = event_channel();
        self.send((op, event))?;
        Ok(source)
    }
}

pub type SubscribeSource<T, U> = EventSource<(T, EventSender<U>)>;
