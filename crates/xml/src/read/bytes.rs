use std::io;
use std::io::Read;

/// A wrapper around a `std::io::Bytes` iterator providing XML newline
/// normalization. Also provides a secondary buffer of bytes to be filled by the
/// user which takes priority over the input stream. This stream is assumed to
/// already be newline normalized.
pub struct Bytes<R: Read> {
    iter: io::Bytes<R>,
    /// the last byte read from the input stream
    last_byte: Option<u8>,
    newline: bool,
    line: usize,
    /// a buffer for ungotten characters
    buf: Vec<u8>,
    // XXX pos won't work unless we track unicode chars
    // pos: usize,
}

impl<R: Read> Iterator for Bytes<R> {
    type Item = Result<u8, io::Error>;

    /// get the next byte from the stream
    fn next(&mut self) -> Option<Result<u8, io::Error>> {
        if let Some(b) = self.buf.pop() {
            return Some(Ok(b));
        }

        if self.newline {
            self.newline = false;
            self.line += 1;
        }

        match self.iter.next() {
            Some(Ok(b)) => {
                // newline normalization
                let ret = match b {
                    b'\n' if self.last_byte == Some(b'\r') => {
                        // skip this char completely
                        self.last_byte = Some(b'\n');
                        return self.next();
                    }
                    b'\r' | b'\n' => {
                        self.newline = true;
                        b'\n'
                    }
                    b => b,
                };

                self.last_byte = Some(b);
                Some(Ok(ret))
            }
            n => n,
        }
    }
}

impl<R: Read> Bytes<R> {
    /// create from a reader stream
    pub fn from_reader(reader: R) -> Self {
        Bytes {
            iter: reader.bytes(),
            last_byte: None,
            newline: false,
            line: 1,
            buf: Vec::new(),
        }
    }

    /// get the current location in the stream
    pub fn location(&self) -> usize {
        self.line
    }

    /// unget a byte
    pub fn unget(&mut self, b: u8) {
        self.buf.push(b);
    }

    /// unget a bunch of bytes
    pub fn unget_buf(&mut self, buf: &[u8]) {
        self.buf.reserve(buf.len());
        for b in buf.iter().rev() {
            self.buf.push(*b);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn read_text() {
        let text = "hello";
        let mut reader = Bytes::from_reader(text.as_bytes());
        assert_eq!(reader.next().unwrap().unwrap(), b'h');
        assert_eq!(reader.next().unwrap().unwrap(), b'e');
        assert_eq!(reader.next().unwrap().unwrap(), b'l');
        assert_eq!(reader.next().unwrap().unwrap(), b'l');
        assert_eq!(reader.next().unwrap().unwrap(), b'o');
        assert!(reader.next().is_none());
    }

    #[test]
    fn unget_bytes() {
        let text = "ha";
        let mut reader = Bytes::from_reader(text.as_bytes());
        assert_eq!(reader.next().unwrap().unwrap(), b'h');
        assert_eq!(reader.next().unwrap().unwrap(), b'a');
        reader.unget(b'a');
        assert_eq!(reader.next().unwrap().unwrap(), b'a');
        reader.unget(b'a');
        assert_eq!(reader.next().unwrap().unwrap(), b'a');
        reader.unget(b'a');
        assert_eq!(reader.next().unwrap().unwrap(), b'a');
        reader.unget(b'a');
        assert_eq!(reader.next().unwrap().unwrap(), b'a');
        assert!(reader.next().is_none());
    }

    #[test]
    fn unget_buf() {
        let text = "";
        let mut reader = Bytes::from_reader(text.as_bytes());
        reader.unget_buf("hello".as_bytes());
        assert_eq!(reader.next().unwrap().unwrap(), b'h');
        assert_eq!(reader.next().unwrap().unwrap(), b'e');
        assert_eq!(reader.next().unwrap().unwrap(), b'l');
        assert_eq!(reader.next().unwrap().unwrap(), b'l');
        assert_eq!(reader.next().unwrap().unwrap(), b'o');
        assert!(reader.next().is_none());
    }

    #[test]
    fn newline_normalization() {
        let text = "\na\r\nb\n\rc\r";
        let mut reader = Bytes::from_reader(text.as_bytes());
        assert_eq!(reader.next().unwrap().unwrap(), b'\n');
        assert_eq!(reader.next().unwrap().unwrap(), b'a');
        assert_eq!(reader.next().unwrap().unwrap(), b'\n');
        assert_eq!(reader.next().unwrap().unwrap(), b'b');
        assert_eq!(reader.next().unwrap().unwrap(), b'\n');
        assert_eq!(reader.next().unwrap().unwrap(), b'\n');
        assert_eq!(reader.next().unwrap().unwrap(), b'c');
        assert_eq!(reader.next().unwrap().unwrap(), b'\n');
        assert!(reader.next().is_none());
    }
}
