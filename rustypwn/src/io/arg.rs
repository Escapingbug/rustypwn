use std::time::Duration;
use rustypwn_derive::ActionArg;

pub type Timeout = Option<Duration>;

// TODO get rid of Vec<u8>, use &[u8] instead
// but this will require modification on lifetimes (especially in derive impl)
/// IO action
#[derive(Debug, Clone, ActionArg)]
pub enum Action {
    Send {
        #[default = "None"]
        timeout: Timeout,
        #[default = "b\"\".to_vec()"]
        content: Vec<u8>,
    },
    Recv {
        #[default = "None"]
        timeout: Timeout,
        #[default = "0x1000"]
        size: usize,
        #[default = "false"]
        /// if we have to receive such size to return
        must: bool,
    },
    Recvline {
        #[default = "None"]
        timeout: Timeout,
    },
    Recvuntil {
        #[default = "None"]
        timeout: Timeout,
        #[default = "\"\".to_string()"]
        pattern: String,
    },
    Sendline {
        #[default = "None"]
        timeout: Timeout,
        #[default = "b\"\".to_vec()"]
        content: Vec<u8>,
    },
    Interactive,
    Shutdown {
        #[default = "true"]
        stdin: bool,
        #[default = "true"]
        stdout: bool,
    }
}

#[test]
fn test_action() {
    use Action::*;

    match send().into() {
        Send {
            timeout,
            ..
        } => {
            assert_eq!(timeout, None);
        },
        _ => {}
    }
}
