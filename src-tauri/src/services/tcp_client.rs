use std::net::TcpStream;
use lazy_static::lazy_static;

struct TcpClientInstance{
    label: String,
    server_host: String,
    stream: Option<TcpStream>,
}

impl TcpClientInstance{
    pub fn new(label: &str, server_host: &str) -> TcpClientInstance{
        TcpClientInstance{
            label: label.into(),
            server_host: server_host.into(),
            stream: None,
        }
    }
    pub fn connect(&mut self) -> Result<(), Box<dyn std::error::Error>>{
        let stream = TcpStream::connect(&self.server_host)?;
        self.stream = Some(stream);
        Ok(())
    }
}