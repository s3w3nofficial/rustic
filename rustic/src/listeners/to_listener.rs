use super::{parsed_listener::ParsedListener, tcp_listener::TcpListener, Listener};
use async_std::io;
use http_types::Url;
use std::net::ToSocketAddrs;

pub trait ToListener {
    type Listener: Listener;

    fn to_listener(self) -> io::Result<Self::Listener>;
}

impl ToListener for Url {
    type Listener = ParsedListener;

    fn to_listener(self) -> io::Result<Self::Listener> {
        match self.scheme() {
            "tcp" | "http" => Ok(ParsedListener::Tcp(TcpListener::from_addrs(
                self.socket_addrs(|| Some(80))?,
            ))),

            "tls" | "ssl" | "https" => Err(io::Error::new(
                io::ErrorKind::Other,
                "parsing TLS listeners not supported yet",
            )),

            _ => Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "unrecognized url scheme",
            )),
        }
    }
}

impl ToListener for String {
    type Listener = ParsedListener;
    fn to_listener(self) -> io::Result<Self::Listener> {
        ToListener::to_listener(self.as_str())
    }
}

impl ToListener for &String {
    type Listener = ParsedListener;
    fn to_listener(self) -> io::Result<Self::Listener> {
        ToListener::to_listener(self.as_str())
    }
}

impl ToListener for &str {
    type Listener = ParsedListener;

    fn to_listener(self) -> io::Result<Self::Listener> {
        if let Ok(socket_addrs) = self.to_socket_addrs() {
            Ok(ParsedListener::Tcp(TcpListener::from_addrs(
                socket_addrs.collect(),
            )))
        } else if let Ok(url) = Url::parse(self) {
            ToListener::to_listener(url)
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("unable to parse listener from `{}`", self),
            ))
        }
    }
}
