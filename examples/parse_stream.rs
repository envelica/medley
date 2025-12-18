use medley::ebnf::{grammar, parse, ParseEvent};
use std::io::BufRead;
use std::time::Instant;

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

fn main() {
    let grammar = grammar! {
        start = digit+;
        digit = [0-9];
    };
    let data = "1".repeat(1024 * 1024); // 1MB
    
    // Benchmark with 64-byte chunks
    let iterations = 10;
    let start = Instant::now();
    for _ in 0..iterations {
        let reader = ChunkedBuf { data: data.as_bytes(), pos: 0, chunk: 64 };
        let events: usize = parse(&grammar, reader)
            .filter(|e| matches!(e, ParseEvent::Token { .. }))
            .count();
        std::hint::black_box(events);
    }
    let elapsed = start.elapsed();
    println!("parse_stream_1mb_chunk64: {} iterations in {:?}", iterations, elapsed);
    println!("  Average: {:?} per iteration", elapsed / iterations);
    
    // Benchmark with 4KB chunks
    let start = Instant::now();
    for _ in 0..iterations {
        let reader = ChunkedBuf { data: data.as_bytes(), pos: 0, chunk: 4096 };
        let events: usize = parse(&grammar, reader)
            .filter(|e| matches!(e, ParseEvent::Token { .. }))
            .count();
        std::hint::black_box(events);
    }
    let elapsed = start.elapsed();
    println!("parse_stream_1mb_chunk4k: {} iterations in {:?}", iterations, elapsed);
    println!("  Average: {:?} per iteration", elapsed / iterations);
}
