use medley::ebnf::{ParseEvent, grammar, parse};
use std::io::BufRead;

struct ChunkedBuf<'a> {
    data: &'a [u8],
    pos: usize,
    chunk: usize,
}

impl<'a> BufRead for ChunkedBuf<'a> {
    fn fill_buf(&mut self) -> std::io::Result<&[u8]> {
        if self.pos >= self.data.len() {
            return Ok(&[]);
        }
        let end = (self.pos + self.chunk).min(self.data.len());
        Ok(&self.data[self.pos..end])
    }
    fn consume(&mut self, amt: usize) {
        self.pos = (self.pos + amt).min(self.data.len());
    }
}

impl<'a> std::io::Read for ChunkedBuf<'a> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        let available = self.fill_buf()?;
        let len = available.len().min(buf.len());
        buf[..len].copy_from_slice(&available[..len]);
        self.consume(len);
        Ok(len)
    }
}

#[test]
fn large_stream_bounded_chunking() {
    let grammar = grammar! {
        start ::= digit { digit };
        digit ::= '0'..'9';
    };
    let data = "1".repeat(2 * 1024 * 1024); // 2MB
    let reader = ChunkedBuf {
        data: data.as_bytes(),
        pos: 0,
        chunk: 128,
    };

    let token_count = parse(&grammar, reader)
        .filter(|e| matches!(e, ParseEvent::Token { .. }))
        .count();

    assert_eq!(token_count, data.len());
}
