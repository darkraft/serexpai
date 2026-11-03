//! Decoder-owned heap during `read_whole_file_raw`.
//!
//! Count the peak of allocations made by the decoder. Exclude the caller-owned
//! input, the sink, and bytes allocated while the CRC fixture is built.
//! The peak for N = 0 and N = 1 MiB is at most 64 KiB, and the larger input
//! does not raise the peak.

use std::alloc::{GlobalAlloc, Layout, System};
use std::io::{self, Read};
use std::sync::atomic::{AtomicUsize, Ordering};

use sxp_kernel::{Crc32, encode_whole_file_raw_header, read_file, read_whole_file_raw};

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

struct Stream {
    header: [u8; 7],
    header_len: usize,
    header_pos: usize,
    zeros_left: u64,
    crc: [u8; 4],
    crc_pos: usize,
}

impl Read for Stream {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut written = 0;
        while written < buf.len() {
            if self.header_pos < self.header_len {
                buf[written] = self.header[self.header_pos];
                self.header_pos += 1;
                written += 1;
                continue;
            }
            if self.zeros_left > 0 {
                let room = buf.len() - written;
                let n64 = self
                    .zeros_left
                    .min(u64::try_from(room).unwrap_or(self.zeros_left));
                let n = usize::try_from(n64).unwrap_or(room);
                for slot in &mut buf[written..written + n] {
                    *slot = 0;
                }
                self.zeros_left -= u64::try_from(n).unwrap_or(0);
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
    let header = encode_whole_file_raw_header(n).unwrap();
    let header_len = usize::from(header.len);
    let crc = crc_zeros(n).to_le_bytes();
    let mut stream = Stream {
        header: header.bytes,
        header_len,
        header_pos: 0,
        zeros_left: n,
        crc,
        crc_pos: 0,
    };
    let base = CURRENT.load(Ordering::Relaxed);
    PEAK.store(base, Ordering::Relaxed);
    let mut sink = io::sink();
    if via_file {
        read_file(&mut stream, &mut sink).unwrap();
    } else {
        read_whole_file_raw(&mut stream, &mut sink).unwrap();
    }
    PEAK.load(Ordering::Relaxed).saturating_sub(base)
}

#[test]
fn decoder_heap_does_not_grow_with_n() {
    for via_file in [false, true] {
        let small = measure_with(0, via_file);
        let large = measure_with(1_048_576, via_file);
        assert!(small <= 65_536, "N=0 peak {small} file={via_file}");
        assert!(large <= 65_536, "N=1MiB peak {large} file={via_file}");
        assert!(
            large <= small,
            "peak grew from {small} to {large} file={via_file}"
        );
    }
}
