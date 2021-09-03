use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::sync::{Arc, Mutex};
use async_std::task;
use futures::stream::StreamExt;
use futures::Future;
use std::fmt::Write;
use std::net::SocketAddrV4;
use tracing::{debug, info};

use crate::error::Error;
use crate::OperationMode;

pub(crate) struct Socket<R>(Arc<Mutex<fn(Vec<u8>) -> R>>)
where
    R: Future<Output = Vec<u8>> + Send + Sync + 'static;

impl<R: futures::Future<Output = Vec<u8>> + Send + Sync + 'static> Socket<R> {
    pub(crate) fn new(f: fn(Vec<u8>) -> R) -> Socket<R>
    where
        R: Future<Output = Vec<u8>>,
    {
        Socket(Arc::new(Mutex::new(f)))
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
        info!("connecting to local : {}", local_addrs);
        let local = TcpStream::connect(local_addrs).await?;
        info!("connecting to remote : {}", remote_addrs);
        let remote = TcpStream::connect(remote_addrs).await?;
        let callback = self.0.clone();
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
        info!("listening on local : {}", listener.local_addr()?);

        Ok(listener
            .incoming()
            .for_each_concurrent(/* limit */ None, |stream| async move {
                let stream = stream.unwrap();
                let callback = self.0.clone();

                task::spawn(async move {
                    info!("connecting to remote : {}", remote_addrs);
                    let remote = TcpStream::connect(remote_addrs).await.unwrap();
                    handle_connection(callback, remote, stream).await;
                });
            })
            .await)
    }
}

async fn handle_connection<R>(
    callback: Arc<Mutex<impl Fn(Vec<u8>) -> R>>,
    mut local: TcpStream,
    mut remote: TcpStream,
) where
    R: Future<Output = Vec<u8>> + Send + Sync,
{
    let mut buffer = [0; 1024];
    while let Ok(size) = remote.read(&mut buffer).await {
        if size > 0 {
            let mut hex_bytes = String::with_capacity(2 * size);
            for (i, &byte) in buffer[..size].iter().enumerate() {
                if i % 16 == 0 {
                    std::write!(hex_bytes, "\n").expect("Dumping hex data failed");
                }
                std::write!(hex_bytes, "{:02X} ", byte).expect("Dumping hex data failed");
            }
            debug!(
                "{} -> {} :{}",
                local.local_addr().unwrap(),
                remote.local_addr().unwrap(),
                hex_bytes,
            );
            let response = callback.lock().await((&buffer[..size]).to_vec()).await;
            local.write(response.as_slice()).await.unwrap();
        }
        local.flush().await.unwrap();
    }
}
