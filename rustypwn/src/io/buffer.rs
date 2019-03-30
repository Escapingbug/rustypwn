use crate::error::Result;
use regex::bytes::Regex;
use super::error::TubeError;

/// buffer to store stream data
#[derive(Default, Debug)]
pub struct Buffer {
    data: Vec<u8>,
}

impl Buffer {
    /// append to the tail of the buffer
    pub fn append(&mut self, data: &mut Vec<u8>) {
        self.data.append(data);
    }

    /// prepend to the buffer
    pub fn prepend(&mut self, data: &mut Vec<u8>) {
        let mut data = data.clone();
        data.append(&mut self.data);
        self.data = data;
    }

    pub fn get_until(&mut self, pat: &str) -> Result<Option<Vec<u8>>> {
        let re = Regex::new(pat).map_err(|e| TubeError::from(e))?;
        match re.find(self.data.as_slice()) {
            Some(mat) => {
                let data = self.data.clone();
                let (l, r) = data.split_at(mat.end());
                self.data = r.to_vec();
                Ok(Some(l.to_vec()))
            },
            None => Ok(None)
        }
    }
}
