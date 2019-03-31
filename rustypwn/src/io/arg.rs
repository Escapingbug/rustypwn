use std::time::Duration;
use rustypwn_derive::ActionArg;

#[derive(Debug, PartialEq, Eq)]
pub enum Timeout {
    Infinite,
    Of(Duration),
}

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
        size: u64,
    },
    Recvline {
        #[default = "Timeout::Infinite"]
        timeout: Timeout,
    },
    Recvuntil {
        #[default = "Timeout::Infinite"]
        timeout: Timeout,
        #[default = "b\"\".to_vec()"]
        pattern: Vec<u8>,
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
