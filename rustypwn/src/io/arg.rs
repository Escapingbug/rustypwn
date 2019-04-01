use std::time::Duration;
use rustypwn_derive::ActionArg;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Timeout {
    Infinite,
    Of(Duration),
}

// TODO get rid of Vec<u8>, use &[u8] instead
// but this will require modification on lifetimes (especially in derive impl)
/// IO action
#[derive(Debug, ActionArg)]
pub enum Action {
    Send {
        #[default = "Timeout::Infinite"]
        timeout: Timeout,
        #[default = "b\"\".to_vec()"]
        content: Vec<u8>,
    },
    Recv {
        #[default = "Timeout::Infinite"]
        timeout: Timeout,
        #[default = "0x1000"]
        size: usize,
        #[default = "false"]
        /// if we have to receive such size to return
        must: bool,
    },
    Recvline {
        #[default = "Timeout::Infinite"]
        timeout: Timeout,
    },
    Recvuntil {
        #[default = "Timeout::Infinite"]
        timeout: Timeout,
        #[default = "\"\".to_string()"]
        pattern: String,
    },
    Sendline {
        #[default = "Timeout::Infinite"]
        timeout: Timeout,
        #[default = "b\"\".to_vec()"]
        content: Vec<u8>,
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
            assert_eq!(timeout, Timeout::Infinite);
        },
        _ => {}
    }
}
