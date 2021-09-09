use async_std::channel::{Receiver, Sender};
use async_std::net::{TcpListener, TcpStream};
use async_std::sync::Arc;
use async_std::task;
use futures::stream::StreamExt;
use futures::{AsyncReadExt, AsyncWriteExt};
use std::net::SocketAddrV4;
use tokio::select;
use tracing::{debug, error, info};

use crate::error::Error;
use crate::utils::dump_hex;
use crate::OperationMode;

pub(crate) struct Socket {
    inbound_tx: Arc<Sender<(u32, Vec<u8>)>>,
    outbound_rx: Arc<Receiver<(u32, Vec<u8>)>>,
}

impl Socket {
    pub(crate) fn new(
        inbound_tx: Sender<(u32, Vec<u8>)>,
        outbound_rx: Receiver<(u32, Vec<u8>)>,
    ) -> Socket {
        Socket {
            inbound_tx: Arc::new(inbound_tx),
            outbound_rx: Arc::new(outbound_rx),
        }
    }

    pub(crate) async fn connect(
        self,
        mode: OperationMode,
        local_addrs: SocketAddrV4,
        remote_addrs: SocketAddrV4,
    ) -> Result<(), Error> {
        let inbound_tx = self.inbound_tx;
        let outbound_rx = self.outbound_rx;
        match mode {
            OperationMode::Server => {
                Self::bind_server(inbound_tx, outbound_rx, local_addrs, remote_addrs).await
            }
            OperationMode::Client(_) => {
                Self::bind_local(inbound_tx, outbound_rx, local_addrs, remote_addrs).await
            }
        }
    }

    async fn bind_server(
        inbound_tx: Arc<Sender<(u32, Vec<u8>)>>,
        outbound_rx: Arc<Receiver<(u32, Vec<u8>)>>,
        local_addrs: SocketAddrV4,
        remote_addrs: SocketAddrV4,
    ) -> Result<(), Error> {
        info!("connecting to local : {}", local_addrs);
        let local = TcpStream::connect(local_addrs).await?;
        info!("connecting to remote : {}", remote_addrs);
        let remote = TcpStream::connect(remote_addrs).await?;
        handle_connection(inbound_tx.clone(), outbound_rx.clone(), local, remote).await;
        Ok(())
    }

    async fn bind_local(
        inbound_tx: Arc<Sender<(u32, Vec<u8>)>>,
        outbound_rx: Arc<Receiver<(u32, Vec<u8>)>>,
        local_addrs: SocketAddrV4,
        remote_addrs: SocketAddrV4,
    ) -> Result<(), Error> {
        // Open up a TCP connection and create a URL.
        let listener = TcpListener::bind(local_addrs).await?;
        info!("listening on local : {}", listener.local_addr()?);

        listener
            .incoming()
            .for_each_concurrent(/* limit */ None, |stream| async {
                let stream = stream.unwrap();

                let (i_tx, o_rx) = (inbound_tx.clone(), outbound_rx.clone());

                task::spawn(async move {
                    info!("connecting to remote : {}", remote_addrs);
                    let remote = TcpStream::connect(remote_addrs).await.unwrap();
                    handle_connection(i_tx, o_rx, remote, stream).await;
                });
            })
            .await;
        Ok(())
    }
}

async fn handle_connection(
    inbound_tx: Arc<Sender<(u32, Vec<u8>)>>,
    outbound_rx: Arc<Receiver<(u32, Vec<u8>)>>,
    mut local: TcpStream,
    mut remote: TcpStream,
) {
    let mut buffer = [0; 1024];
    loop {
        select! {
            res = remote.read(&mut buffer) => {
                match res {
                    Ok(size) => {
                        if size > 0 {
                            debug!(
                                "{} -> {} :{}",
                                local.local_addr().unwrap(),
                                remote.local_addr().unwrap(),
                                dump_hex(&buffer[..size]),
                            );

                            if let Err(err) = inbound_tx.send((0, (&buffer[..size]).to_vec())).await {
                                error!("could not send buffer to inbound_tx : {:?}", err);
                            }
                        }
                    },
                    Err(err) => error!("error reading from remote socket : {:?}", err),
                }
            }
            res = outbound_rx.recv() => {
                match res {
                    Ok((_, response)) => {
                        local.write(response.as_slice()).await.unwrap();
                        local.flush().await.unwrap();
                    },
                    Err(err) => error!("error reading from outbound socket : {:?}", err),
                }
            }
        }
    }
}
