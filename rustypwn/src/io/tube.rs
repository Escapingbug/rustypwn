use std::time::SystemTime;
use super::buffer::Buffer;
use super::arg::{
    Action,
    Timeout,
};
use super::error::{Error, ErrorKind};

pub trait TubeInternal {
    /// internal buffer
    fn mut_buffer(&self) -> &mut Buffer;
    fn buffer(&self) -> &Buffer;

    fn send(&mut self, action: Action) -> Result<(), Error>;
    fn recv(&mut self, action: Action) -> Result<Vec<u8>, Error>;

    fn sendline(&mut self, action: Action) -> Result<(), Error> {
        match action {
            Action::Sendline {
                timeout,
                content
            } => {
                let mut content = content;
                content.push(b'\n');
                self.send(Action::Send{
                    timeout: timeout,
                    content: content,
                })
            },
            _ => panic!("Incorrect action, internal bug")
        }
    }

    fn recvuntil(&mut self, action: Action) -> Result<Vec<u8>, Error> {

        match action {
            Action::Recvuntil {
                timeout,
                pattern,
            } => {
                let now = SystemTime::now();
                loop {
                    match timeout {
                        Timeout::Of(timeout) => {
                            match now.elapsed() {
                                Ok(elapsed) => {
                                    if elapsed > timeout {
                                        return Err(Error::from_kind(ErrorKind::Timeout));
                                    }
                                },
                                _ => {
                                    panic!("get elapsed time error, critical internal bug");
                                }
                            }
                        }
                        _ => {},

                    }

                    let res = self.mut_buffer().get_until(&pattern)?;
                    match res {
                        Some(mat) => {
                            return Ok(mat);
                        },
                        _ => {
                            let arg = Action::Recv {
                                timeout: timeout,
                                size: 0x1000
                            };
                            self.recv(arg)?;
                        },
                    }

                }
            },
            _ => panic!("Inccorrect action, internal bug")
        }

    }

    fn recvline(&mut self, action: Action) -> Result<Vec<u8>, Error> {
        match action {
            Action::Recvline {
                timeout,
            } => {
                let arg = Action::Recvuntil {
                    timeout: timeout,
                    pattern: "\n".to_string()
                };
                self.recvuntil(arg)
            },
            _ => panic!("Inccorrect action, internal bug")
        }
        
    }
}

pub trait Tube: TubeInternal {
    fn act<T: Into<Action>>(&mut self, action: T) -> Result<Option<Vec<u8>>, Error> {
        let action = action.into();
        match action {
            Action::Send {..} => self.send(action).map(|_res| None),
            Action::Recv {..} => self.recv(action).map(|res| Some(res)),
            Action::Sendline {..} => self.sendline(action).map(|_res| None),
            Action::Recvline {..} => self.recvline(action).map(|res| Some(res)),
            Action::Recvuntil {..} => self.recvuntil(action).map(|res| Some(res)),
        }
    }
}
