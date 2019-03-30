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

impl Action {

    pub fn send() -> Self {
        Action::Send {
            timeout: Timeout::Infinite,
            content: b"".to_vec()
        }
    }

    pub fn recv() -> Self {
        Action::Recv {
            timeout: Timeout::Infinite,
            size: 0x1000
        }
    }

    pub fn recvline() -> Self {
        Action::Recvline {
            timeout: Timeout::Infinite,
        }
    }

    pub fn recvuntil() -> Self {
        Action::Recvuntil {
            timeout: Timeout::Infinite,
            pattern: b"".to_vec()
        }
    }

    pub fn timeout(mut self, new_timeout: Timeout) -> Self {
        use Action::*;

        match self {
            Send { 
                ref mut timeout,
                ..
            } => {
                *timeout = new_timeout;
            },

            Sendline {
                ref mut timeout,
                ..
            } => {
                *timeout = new_timeout;
            },

            Recv {
                ref mut timeout,
                ..
            } => {
                *timeout = new_timeout;
            },

            Recvline {
                ref mut timeout,
            } => {
                *timeout = new_timeout;
            },

            Recvuntil {
                ref mut timeout,
                ..
            } => {
                *timeout = new_timeout;
            },

        }

        self
    }
}


#[test]
fn test_action() {
    use Action::*;

    match Action::recv() {
        Recv {
            timeout,
            ..
        } => {
            assert_eq!(timeout, Timeout::Infinite);
        },
        _ => {
            panic!("Incorrect action type");
        }
    }


    match Action::recv().timeout(Timeout::Of(Duration::from_secs(3))) {
        Recv {
            timeout,
            ..
        } => {
            assert_eq!(timeout, Timeout::Of(Duration::from_secs(3)));
        },
        _ => {
            panic!("Incorrect action type");
        }
    }
}
