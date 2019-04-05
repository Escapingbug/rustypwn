use super::arg::{Action, Timeout};
use super::buffer::Buffer;
use super::error::{Error, ErrorKind};
use super::tube::{Tube, TubeInternal};
use rustypwn_derive::action;
use std::ffi::OsString;
use std::ops::Drop;
use std::time::SystemTime;
use subprocess::{Popen, PopenConfig, Redirection};

pub struct ProcessArg<'a> {
    argv: &'a [&'a str],
    env: Option<Vec<(&'a str, &'a str)>>,
}

impl<'a> Default for ProcessArg<'a> {
    fn default() -> Self {
        ProcessArg {
            argv: &[""],
            env: None,
        }
    }
}

impl<'a> ProcessArg<'a> {
    pub fn argv(mut self, new_argv: &'a [&'a str]) -> Self {
        self.argv = new_argv;
        self
    }

    pub fn env(mut self, environ: Vec<(&'a str, &'a str)>) -> Self {
        self.env = Some(environ);
        self
    }
}

pub struct Process {
    buf: Buffer,
    p: Popen,
}

impl Process {
    pub fn try_new<'a>(arg: ProcessArg<'a>) -> Result<Self, Error> {
        let env = match arg.env {
            Some(env) => Some(
                env.iter()
                    .map(|each| (each.0.to_string().into(), each.1.to_string().into()))
                    .collect::<Vec<(OsString, OsString)>>(),
            ),
            None => None,
        };
        let p = Popen::create(
            arg.argv,
            PopenConfig {
                stdin: Redirection::Pipe,
                stdout: Redirection::Pipe,
                stderr: Redirection::Pipe,
                detached: true,
                env: env,
                ..Default::default()
            },
        )?;
        let buf = Buffer::default();
        Ok(Self { buf: buf, p: p })
    }
}

impl TubeInternal for Process {
    fn mut_buffer(&mut self) -> &mut Buffer {
        &mut self.buf
    }

    fn buffer(&self) -> &Buffer {
        &self.buf
    }

    #[action(stdin, stdout)]
    fn shutdown(&mut self, action: Action) -> Result<(), Error> {
        if stdin {
            self.p.shutdown_stdin()?;
        }

        if stdout {
            self.p.shutdown_stdout()?;
        }

        Ok(())
    }

    #[action(timeout, content)]
    fn send(&mut self, action: Action) -> Result<(), Error> {
        let _ = timeout;
        if let Some(exit) = self.p.poll() {
            return Err(Error::from_kind(ErrorKind::UnexpectedTerminate(exit)));
        }

        let input = Some(content.as_ref());
        let (out, err) = self.p.communicate_bytes(input)?;
        // the time seq of stdout and stderr is not known naturally
        // so we just arrange them in this way
        if let Some(mut out) = out {
            self.mut_buffer().append(&mut out);
        }

        if let Some(mut err) = err {
            self.mut_buffer().append(&mut err);
        }

        Ok(())
    }

    fn recv_once(&mut self, size: usize, timeout: Timeout) -> Result<Vec<u8>, Error> {
        let mut v = vec![];
        let now = SystemTime::now();
        loop {
            let (out, err) = self.p.communicate_bytes(None)?;
            if let Some(mut out) = out {
                v.append(&mut out);
            }

            if let Some(mut err) = err {
                v.append(&mut err);
            }
            if v.len() >= size {
                break;
            }

            if let Some(duration) = timeout {
                if now.elapsed().unwrap() >= duration {
                    break;
                }
            }
        }

        Ok(v)
    }

    #[action(timeout, size, must)]
    fn recv(&mut self, action: Action) -> Result<Vec<u8>, Error> {
        let now = SystemTime::now();

        loop {
            let res = self.mut_buffer().get(size, must);
            if let Some(res) = res {
                return Ok(res);
            }

            if let Some(exit) = self.p.poll() {
                return Err(Error::from_kind(ErrorKind::UnexpectedTerminate(exit)));
            }

            let (out, err) = self.p.communicate_bytes(None)?;
            if let Some(mut out) = out {
                self.mut_buffer().append(&mut out);
            }
            if let Some(mut err) = err {
                self.mut_buffer().append(&mut err);
            }

            if let Some(timeout) = timeout {
                match now.elapsed() {
                    Ok(elapsed) => {
                        if elapsed >= timeout {
                            return Err(Error::timeout());
                        }
                    }
                    _ => panic!("get time error, internal bug"),
                }
            }
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        self.p.terminate().expect("unable to terminate process")
    }
}

impl Tube for Process {}

#[cfg(unix)]
#[test]
fn popen_test_unix() {
    use super::arg::*;
    use std::time::*;

    let mut p = Process::try_new(ProcessArg::default().argv(&["cat"])).unwrap();
    p.send(send().content(b"123".to_vec()).into()).unwrap();
    let res = p.recv(recv().size(20).into()).unwrap();
    assert!(res == b"123");

    let mut p = Process::try_new(ProcessArg::default().argv(&["cat"])).unwrap();
    let now = SystemTime::now();
    assert!(
        p.recv(
            recv()
                .size(20)
                .timeout(Some(Duration::from_secs(1)))
                .into()
        )
        .is_err()
            == true
    );
    let elapsed = now.elapsed().unwrap();
    assert!(Duration::from_secs(2) >= elapsed);
    assert!(Duration::from_secs(1) <= elapsed);
    let p = Process::try_new(ProcessArg::default().argv(&["bash"])).unwrap();
    drop(p);
}
