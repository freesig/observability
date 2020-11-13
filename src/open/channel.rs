use crate::MsgWrap;
use derive_more::{From, Into};
use shrinkwraprs::Shrinkwrap;

pub mod mpsc {
    use super::*;

    #[derive(From, Into, Shrinkwrap)]
    #[shrinkwrap(mutable)]
    #[shrinkwrap(unsafe_ignore_visibility)]
    pub struct Sender<T>(tokio::sync::mpsc::Sender<MsgWrap<T>>);
    #[derive(From, Into, Shrinkwrap)]
    #[shrinkwrap(mutable)]
    #[shrinkwrap(unsafe_ignore_visibility)]
    pub struct Receiver<T>(tokio::sync::mpsc::Receiver<MsgWrap<T>>);

    pub fn channel<T>(buffer: usize) -> (Sender<T>, Receiver<T>) {
        let (tx, rx) = tokio::sync::mpsc::channel(buffer);
        (tx.into(), rx.into())
    }

    impl<T> Clone for Sender<T> {
        fn clone(&self) -> Self {
            Self(self.0.clone())
        }
    }

    impl<T> Sender<T> {
        pub async fn send(
            &mut self,
            value: T,
        ) -> Result<(), tokio::sync::mpsc::error::SendError<T>> {
            self.0
                .send(value.into())
                .await
                .map_err(|e| tokio::sync::mpsc::error::SendError(e.0.without_context()))
        }
    }
    
    impl<T> Receiver<T> {
        pub async fn recv(&mut self) -> Option<T> {
            self.0.recv().await.map(|t| t.inner())
        }
    }
}
