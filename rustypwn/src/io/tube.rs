use super::arg::{Action, Timeout};
use super::buffer::Buffer;
use super::error::{Error, ErrorKind};
use rustypwn_derive::action;
use std::io;
use std::io::{BufRead, Write};
use std::ops::Drop;
use std::time::{Duration, SystemTime};

pub trait TubeInternal: Drop {
    /// internal buffer
    fn mut_buffer(&mut self) -> &mut Buffer;
    fn buffer(&self) -> &Buffer;

    fn send(&mut self, action: Action) -> Result<(), Error>;
    fn recv(&mut self, action: Action) -> Result<Vec<u8>, Error>;
    fn shutdown(&mut self, action: Action) -> Result<(), Error>;

    #[action(timeout, content)]
    fn sendline(&mut self, action: Action) -> Result<(), Error> {
        let mut content = content;
        content.push(b'\n');
        self.send(Action::Send {
            timeout: timeout,
            content: content,
        })
    }

    #[action(timeout, pattern)]
    fn recvuntil(&mut self, action: Action) -> Result<Vec<u8>, Error> {
        let now = SystemTime::now();
        loop {
            match timeout {
                Timeout::Of(timeout) => match now.elapsed() {
                    Ok(elapsed) => {
                        if elapsed > timeout {
                            return Err(Error::from_kind(ErrorKind::Timeout));
                        }
                    }
                    _ => {
                        panic!("get elapsed time error, critical internal bug");
                    }
                },
                _ => {}
            }

            let res = self.mut_buffer().get_until(&pattern)?;
            match res {
                Some(mat) => {
                    return Ok(mat);
                }
                _ => {
                    let arg = Action::Recv {
                        timeout: timeout,
                        size: 0x1000,
                        must: false,
                    };
                    self.recv(arg)?;
                }
            }
        }
    }

    #[action(timeout)]
    fn recvline(&mut self, action: Action) -> Result<Vec<u8>, Error> {
        let arg = Action::Recvuntil {
            timeout: timeout,
            pattern: "\n".to_string(),
        };
        self.recvuntil(arg)
    }

    #[action]
    fn interactive(&mut self, action: Action) -> Result<(), Error> {
        let stdin = io::stdin();
        print!("$ ");
        io::stdout().flush().unwrap();
        for line in stdin.lock().lines() {
            let line = line.expect("unable to read line in interactive");
            let arg = Action::Sendline {
                timeout: Timeout::Infinite,
                content: line.as_bytes().to_vec(),
            };
            self.sendline(arg)?;
            let arg = Action::Recv {
                timeout: Timeout::Of(Duration::from_millis(50)),
                size: 0x1000,
                must: false,
            };
            let mut recved = Vec::new();
            loop {
                let res = self.recv(arg.clone());
                match res {
                    Ok(res) => recved.extend(res),
                    Err(e) if e.kind == ErrorKind::Timeout => {
                        break;
                    }
                    Err(e) => return Err(e),
                }
            }
            print!("{}", recved.iter().map(|c| *c as char).collect::<String>());
            print!("$ ");
            io::stdout().flush().unwrap();
        }

        self.shutdown(Action::Shutdown {
            stdin: true,
            stdout: true,
        })?;
        Ok(())
    }
}

pub trait Tube: TubeInternal {
    fn act<T: Into<Action>>(&mut self, action: T) -> Result<Option<Vec<u8>>, Error> {
        let action = action.into();
        match action {
            Action::Send { .. } => self.send(action).map(|_res| None),
            Action::Recv { .. } => self.recv(action).map(|res| Some(res)),
            Action::Sendline { .. } => self.sendline(action).map(|_res| None),
            Action::Recvline { .. } => self.recvline(action).map(|res| Some(res)),
            Action::Recvuntil { .. } => self.recvuntil(action).map(|res| Some(res)),
            Action::Interactive => self.interactive(action).map(|_res| None),
            Action::Shutdown { .. } => self.shutdown(action).map(|_res| None),
        }
    }
}
