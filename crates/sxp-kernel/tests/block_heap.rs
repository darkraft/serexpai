//! Decoder-owned heap during `read_blocked_file`.
//!
//! Count the peak of allocations made by the decoder. Exclude the caller-owned
//! input, the sink, and bytes allocated while the zero-file fixture is built.
//! The peak for one 256-byte Raw Block and for N = 1 MiB of those Blocks is at
//! most 64 KiB, and the larger input does not raise the peak.

use std::alloc::{GlobalAlloc, Layout, System};
use std::io::{self, Read};
use std::sync::atomic::{AtomicUsize, Ordering};

use sxp_kernel::{Crc32, encode_leb128, read_blocked_file, read_file};

struct Counting;

static CURRENT: AtomicUsize = AtomicUsize::new(0);
static PEAK: AtomicUsize = AtomicUsize::new(0);

fn note_alloc(size: usize) {
    let prev = CURRENT.fetch_add(size, Ordering::Relaxed);
    let now = prev.saturating_add(size);
    let mut peak = PEAK.load(Ordering::Relaxed);
    while now > peak {
        match PEAK.compare_exchange_weak(peak, now, Ordering::Relaxed, Ordering::Relaxed) {
            Ok(_) => break,
            Err(seen) => peak = seen,
        }
    }
}

fn note_free(size: usize) {
    CURRENT.fetch_sub(size, Ordering::Relaxed);
}

unsafe impl GlobalAlloc for Counting {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let ptr = unsafe { System.alloc(layout) };
        if !ptr.is_null() {
            note_alloc(layout.size());
        }
        ptr
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        note_free(layout.size());
        unsafe { System.dealloc(ptr, layout) }
    }

    unsafe fn realloc(&self, ptr: *mut u8, layout: Layout, new_size: usize) -> *mut u8 {
        let new_ptr = unsafe { System.realloc(ptr, layout, new_size) };
        if !new_ptr.is_null() {
            note_free(layout.size());
            note_alloc(new_size);
        }
        new_ptr
    }
}

#[global_allocator]
static ALLOC: Counting = Counting;

fn crc_zeros(n: u64) -> u32 {
    let mut crc = Crc32::new();
    let zeros = [0u8; 8192];
    let mut left = n;
    while left > 0 {
        let chunk = usize::try_from(left.min(8192)).unwrap_or(8192);
        crc.update(&zeros[..chunk]);
        left -= u64::try_from(chunk).unwrap_or(0);
    }
    crc.finish()
}

struct RawZeros {
    header: Vec<u8>,
    header_pos: usize,
    left: u64,
    block_left: u64,
    crc: [u8; 4],
    crc_pos: usize,
}

impl Read for RawZeros {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut written = 0usize;
        while written < buf.len() {
            if self.header_pos < self.header.len() {
                buf[written] = self.header[self.header_pos];
                self.header_pos += 1;
                written += 1;
                continue;
            }
            if self.left > 0 && self.block_left == 0 {
                buf[written] = 0x00;
                self.block_left = self.left.min(256);
                written += 1;
                continue;
            }
            if self.block_left > 0 {
                let room = buf.len() - written;
                let n64 = self
                    .block_left
                    .min(u64::try_from(room).unwrap_or(self.block_left));
                let n = usize::try_from(n64).unwrap_or(room);
                for slot in &mut buf[written..written + n] {
                    *slot = 0;
                }
                self.block_left -= u64::try_from(n).unwrap_or(0);
                self.left -= u64::try_from(n).unwrap_or(0);
                written += n;
                continue;
            }
            if self.crc_pos < 4 {
                buf[written] = self.crc[self.crc_pos];
                self.crc_pos += 1;
                written += 1;
                continue;
            }
            break;
        }
        Ok(written)
    }
}

fn measure_with(n: u64, via_file: bool) -> usize {
    let leb = encode_leb128(n).expect("length");
    let mut header = vec![0x8D, 0x01];
    header.extend_from_slice(&leb.bytes[..usize::from(leb.len)]);
    let crc = crc_zeros(n).to_le_bytes();
    let mut stream = RawZeros {
        header,
        header_pos: 0,
        left: n,
        block_left: 0,
        crc,
        crc_pos: 0,
    };
    let base = CURRENT.load(Ordering::Relaxed);
    PEAK.store(base, Ordering::Relaxed);
    let mut sink = io::sink();
    if via_file {
        read_file(&mut stream, &mut sink).expect("read");
    } else {
        read_blocked_file(&mut stream, &mut sink).expect("read");
    }
    PEAK.load(Ordering::Relaxed).saturating_sub(base)
}

#[test]
fn decoder_heap_does_not_grow_with_n() {
    for via_file in [false, true] {
        let small = measure_with(256, via_file);
        let large = measure_with(1_048_576, via_file);
        assert!(small <= 65_536, "N=256 peak {small} file={via_file}");
        assert!(large <= 65_536, "N=1MiB peak {large} file={via_file}");
        assert!(
            large <= small,
            "peak grew from {small} to {large} file={via_file}"
        );
    }
}
