use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;
use futures::stream::StreamExt;
use futures::Future;
use std::net::SocketAddrV4;

use crate::error::Error;
use crate::OperationMode;
use std::sync::Arc;

pub(crate) struct Socket<R>(Arc<fn(Vec<u8>) -> R>)
where
    R: Future<Output = Vec<u8>> + Send + Sync + 'static;

impl<R: futures::Future<Output = Vec<u8>> + Send + Sync + 'static> Socket<R> {
    pub(crate) fn new(f: fn(Vec<u8>) -> R) -> Socket<R>
    where
        R: Future<Output = Vec<u8>>,
    {
        Socket(Arc::new(f))
    }

    pub(crate) async fn connect(
        self,
        mode: OperationMode,
        local_addrs: SocketAddrV4,
        remote_addrs: SocketAddrV4,
    ) -> Result<(), Error> {
        match mode {
            OperationMode::Server => self.bind_server(local_addrs, remote_addrs).await,
            OperationMode::Client(_) => self.bind_local(local_addrs, remote_addrs).await,
        }
    }

    async fn bind_server(
        &self,
        local_addrs: SocketAddrV4,
        remote_addrs: SocketAddrV4,
    ) -> Result<(), Error> {
        eprintln!("connecting to local : {}", local_addrs);
        let local = TcpStream::connect(local_addrs).await?;
        eprintln!("connecting to remote : {}", remote_addrs);
        let remote = TcpStream::connect(remote_addrs).await?;
        let callback = Arc::clone(&self.0);
        handle_connection(callback, local, remote).await;
        Ok(())
    }

    async fn bind_local(
        &self,
        local_addrs: SocketAddrV4,
        remote_addrs: SocketAddrV4,
    ) -> Result<(), Error> {
        // Open up a TCP connection and create a URL.
        let listener = TcpListener::bind(local_addrs).await?;
        eprintln!("listening on local : {}", listener.local_addr()?);

        Ok(listener
            .incoming()
            .for_each_concurrent(/* limit */ None, |stream| async move {
                let stream = stream.unwrap();
                let callback = Arc::clone(&self.0);

                task::spawn(async move {
                    eprintln!("connecting to remote : {}", remote_addrs);
                    let remote = TcpStream::connect(remote_addrs).await.unwrap();
                    handle_connection(callback, stream, remote).await;
                });
            })
            .await)
    }
}

async fn handle_connection<R>(
    callback: Arc<impl Fn(Vec<u8>) -> R>,
    mut local: TcpStream,
    mut remote: TcpStream,
) where
    R: Future<Output = Vec<u8>> + Send + Sync,
{
    let mut buffer = [0; 1024];
    while let Ok(_size) = local.read(&mut buffer).await {
        remote
            .write(callback(buffer.to_vec()).await.as_slice())
            .await
            .unwrap();
        remote.flush().await.unwrap();
    }
}
