use crate::submit::Receiver;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub struct Null;

impl Null {
    pub async fn submit_loop<T, U>(self, mut receiver: Receiver<T, U>) -> crate::Result<()>
    where
        U: Default,
    {
        while let Some((_, reply)) = receiver.recv().await {
            reply
                .send(Default::default())
                .map_err(|_| crate::err!("unexpected reply channel closing"))?
        }
        Ok(())
    }
}
