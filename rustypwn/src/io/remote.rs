use rustypwn_derive::action;

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::{SystemTime, Duration};
use super::error::Error;
use super::buffer::Buffer;
use super::tube::{TubeInternal, Tube};
use super::arg::{Action, Timeout};

#[derive(Debug)]
pub struct RemoteArg {
    addr: SocketAddr,
    timeout: Timeout,
}

impl RemoteArg {
    pub fn new<T: ToSocketAddrs>(addr_repr: T) -> Self {
        Self {
            addr: addr_repr.to_socket_addrs().unwrap().next().unwrap(),
            timeout: None,
        }
    }

    pub fn timeout(mut self, new_timeout: Timeout) -> Self {
        self.timeout = new_timeout;
        self
    }
}

pub struct Remote {
    buf: Buffer,
    stream: TcpStream,
}

impl Remote {
    pub fn try_new(arg: RemoteArg) -> Result<Self, Error> {
        let stream = TcpStream::connect(arg.addr)?;
        stream.set_nonblocking(false)?;
        Ok(Self {
            buf: Buffer::default(),
            stream: stream
        })
    }
}

impl Drop for Remote {
    fn drop(&mut self) {
        std::mem::drop(&mut self.stream)
    }
}

impl TubeInternal for Remote {
    fn mut_buffer(&mut self) -> &mut Buffer {
        &mut self.buf
    }

    fn buffer(&self) -> &Buffer {
        &self.buf
    }

    #[action(timeout, content)]
    fn send(&mut self, action: Action) -> Result<(), Error> {
        self.stream.set_write_timeout(timeout)?;
        self.stream.write(&content)
            .map_err(|e| if e.kind() == std::io::ErrorKind::TimedOut { Error::timeout() } else { e.into() })?;
        Ok(())
    }

    fn recv_once(&mut self, size: usize, timeout: Option<Duration>) -> Result<Vec<u8>, Error> {
        let mut content = Vec::with_capacity(size);
        content.resize(size, 0u8);

        self.stream.set_read_timeout(timeout)?;
        let n = self.stream.read(&mut content)
            .map_err(|e| if e.kind() == std::io::ErrorKind::TimedOut { Error::timeout() } else { e.into() })?;
        Ok(content[0..n].to_vec())
    }

    #[action(timeout, size, must)]
    fn recv(&mut self, action: Action) -> Result<Vec<u8>, Error> {
        let now = SystemTime::now();
        let mut content = Vec::with_capacity(size);
        content.resize(size, 0u8);

        self.stream.set_read_timeout(timeout)?;

        loop {
            let res = self.mut_buffer().get(size, must);
            if let Some(res) = res {
                return Ok(res);
            }

            let n = self.stream.read(&mut content)
                .map_err(|e| if e.kind() == std::io::ErrorKind::TimedOut { Error::timeout() } else { e.into() } )?;
            let mut put = content[0..n].to_vec();
            self.mut_buffer().append(&mut put);

            if let Some(timeout) = timeout {
                match now.elapsed() {
                    Ok(elapsed) => {
                        if elapsed >= timeout {
                            return Err(Error::timeout());
                        }
                    },
                    _ => panic!("get time error, internal bug"),
                }
            }
        }
    }

    #[action(stdin, stdout)]
    fn shutdown(&mut self, action: Action) -> Result<(), Error> {
        let shutdown = {
            if stdin && stdout {
                std::net::Shutdown::Both
            } else if stdin {
                std::net::Shutdown::Write
            } else {
                std::net::Shutdown::Read
            }
        };

        Ok(self.stream.shutdown(shutdown)?)
    }
}

impl Tube for Remote {}

#[test]
fn test_remote() {
    use super::arg::*;

    let mut p = Remote::try_new(RemoteArg::new("localhost:11222")).unwrap();
    p.sendline(sendline().content(b"hello".to_vec()).into()).unwrap();
    assert_eq!(&p.recvline(recvline().into()).unwrap(), b"yes\n");
}
