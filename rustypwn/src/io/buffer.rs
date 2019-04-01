use regex::bytes::Regex;
use super::error::Error;

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

    pub fn get_until(&mut self, pat: &str) -> Result<Option<Vec<u8>>, Error> {
        let re = Regex::new(pat).map_err(|e| Error::from_source(Box::new(e)))?;
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

    pub fn get(&mut self, size: usize, strict: bool) -> Option<Vec<u8>> {
        if self.data.len() == 0 {
            return None;
        }

        if self.data.len() >= size {
            let data = self.data.clone();
            let (l, r) = data.split_at(size);
            self.data = r.to_vec();
            Some(l.to_vec())
        } else if strict {
            None 
        } else {
            Some(self.data.clone())
        }
    }
}
