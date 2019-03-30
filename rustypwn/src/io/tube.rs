use std::time::SystemTime;
use crate::error::Result;
use super::buffer::Buffer;
use super::arg::Action;

pub trait Tube {
    /// internal buffer
    fn mut_buffer(&self) -> &mut Buffer;

    fn send(&mut self, mut action_arg: Action) -> Result<()>;
    fn recv(&mut self, mut action_arg: Action) -> Result<Vec<u8>>;

    fn sendline(&mut self, mut action_arg: Action) -> Result<()> {
        let send_action_arg = Action::Send {
            timeout: action_arg.timeout,
            content: action_arg.content + b"\n"
        action_arg.content.push(b'\n');
        self.send(send_action_arg);
    }

    fn recvuntil(&mut self, mut action_arg: Action) -> Result<Vec<u8>> {
        let mut buf = self.mut_buffer();
        let now = SystemTime::now();

        loop {

            match now.elapsed() {
                Ok(elapsed) => {
                    if elapsed > action_arg
                }
            }

            let res = buf.get_until(pattern)?;
            match res {
                Some(mat) => {
                    return Ok(mat);
                },
                _ => {
                    buf.append(&mut self.recv(0x1000)?);
                },
            }
        }
    }

    fn recvline(&mut self) -> Result<Vec<u8>> {
        self.recvuntil(b"\n".to_vec())
    }
}
