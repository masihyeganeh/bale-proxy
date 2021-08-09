use async_std::channel::{unbounded, Receiver, Sender};
use async_std::net::{TcpListener, TcpStream};
use async_std::prelude::*;
use async_std::task;
use lazy_static::lazy_static;
use regex::Regex;
use url::Url;

use crate::error::Error;
use std::collections::HashMap;

const TEMPLATE: &'static str = include_str!("../template/input.html");

pub async fn get_from_web(message: &'static str) -> Result<String, Error> {
    // Open up a TCP connection and create a URL.
    let listener = TcpListener::bind(("127.0.0.1", 8087)).await?;
    let addr = format!("http://{}", listener.local_addr()?);
    println!("Please open {} and fill the form", addr);

    let (sender, receiver): (Sender<String>, Receiver<String>) = unbounded();

    // For each incoming TCP connection, spawn a task and call `accept`.
    let listener_handle = task::spawn(async move {
        let mut incoming = listener.incoming();
        while let Some(stream) = incoming.next().await {
            let stream = stream.unwrap();
            let tx = sender.clone();
            task::spawn(async move {
                match accept(stream, message).await {
                    Ok(res) => {
                        let _ = tx.send(res).await;
                        return;
                    }
                    Err(Error::ServerError(_)) => {}
                    Err(err) => eprintln!("{}", err),
                }
            });
        }
    });
    let res = receiver.recv().await?;
    listener_handle.cancel().await;
    Ok(res)
}

// Take a TCP stream, and convert it into sequential HTTP request / response pairs.
async fn accept(mut stream: TcpStream, title: &'static str) -> Result<String, Error> {
    let mut result: Option<String> = None;
    let mut message = "";

    let mut buf = [0u8; 4096];
    stream.read(&mut buf).await?;
    let req_str = String::from_utf8_lossy(&buf);

    lazy_static! {
        static ref RE: Regex = Regex::new("GET ([^ ]+) HTTP").unwrap();
    }
    let path = RE
        .captures(&req_str)
        .map(|captures| captures.get(1).map(|m| m.as_str()))
        .flatten()
        .ok_or(Error::InternalError(
            "Could not get path from http request".to_string(),
        ))?;
    let url = Url::parse(format!("http://localhost{}", path).as_ref())?;

    let q: HashMap<_, _> = url.query_pairs().into_owned().collect();
    if let Some(input) = q.get("input") {
        result = Some(input.trim().to_string());
        message = "Thanks. Please check result in app output";
    }

    stream
        .write(b"HTTP/1.1 200 OK\r\nContent-Type: text/html; charset=UTF-8\r\n\r\n")
        .await?;
    if url.path() == "/" {
        stream
            .write(
                TEMPLATE
                    .replace("{{message}}", message)
                    .replace("{{title}}", title)
                    .as_ref(),
            )
            .await?;
    }
    stream.write(b"\r\n").await?;

    result.ok_or(Error::ServerError("No input".to_string()))
}
