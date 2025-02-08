#![allow(clippy::missing_safety_doc)]
use concurrent_queue::ConcurrentQueue;
use dashmap::DashMap;
#[cfg(not(target_os = "linux"))]
use memmap2::{MmapMut, MmapOptions};
#[cfg(target_os = "linux")]
use memmap2::{MmapMut, MmapOptions, RemapOptions};
use parking_lot::{Condvar, Mutex};
use std::convert::TryInto;
use std::simd::{cmp::SimdPartialEq, Simd};
use std::sync::atomic::{
    AtomicI16, AtomicI32, AtomicI64, AtomicI8, AtomicU16, AtomicU32, AtomicU8, Ordering,
};
use std::sync::Arc;
use std::time::{Duration, Instant};
use std::{ptr, slice};

const PAGE_SIZE: u32 = 64 * 1024;
const VECTOR_SIZE: usize = 16;

type WaitQueue = Arc<ConcurrentQueue<Arc<WaitEntry>>>;

#[derive(Debug)]
struct WaitEntry {
    condvar: Condvar,
    mutex: Mutex<()>,
}

pub struct LinearMemory {
    memory: MmapMut,
    wait_queues: DashMap<i32, WaitQueue>,
}

impl LinearMemory {
    pub fn new(pages: u32) -> Self {
        let memory = MmapOptions::new()
            .len((pages * PAGE_SIZE) as usize)
            .map_anon()
            .expect("Failed to create memory map");

        Self {
            memory,
            wait_queues: DashMap::new(),
        }
    }

    #[cfg(target_os = "linux")]
    pub fn grow(&mut self, pages: u32) -> bool {
        let additional_size = (pages * PAGE_SIZE) as usize;
        let current_size = self.memory.len();
        let new_size = current_size + additional_size;

        unsafe {
            self.memory
                .remap(new_size, RemapOptions::new().may_move(true))
                .is_ok()
        }
    }

    #[cfg(not(target_os = "linux"))]
    pub fn grow(&mut self, pages: u32) -> bool {
        let additional_size = (pages * PAGE_SIZE) as usize;
        let current_size = self.memory.len();
        let new_size = current_size + additional_size;

        let mut new_memory = memmap2::MmapOptions::new()
            .len(new_size)
            .map_anon()
            .expect("Failed to create a new memory map");

        new_memory[..current_size].copy_from_slice(&self.memory[..current_size]);
        self.memory = new_memory;
        true
    }

    pub fn copy(
        &self,
        src_offset: i32,
        dest_memory: &mut LinearMemory,
        dest_offset: i32,
        byte_count: i32,
    ) {
        let src_ptr = self.memory.as_ptr().wrapping_add(src_offset as usize);
        let dest_ptr = dest_memory
            .memory
            .as_mut_ptr()
            .wrapping_add(dest_offset as usize);

        debug_assert!(
            (src_offset as usize) + (byte_count as usize) <= self.memory.len(),
            "Source range exceeds memory bounds"
        );
        debug_assert!(
            (dest_offset as usize) + (byte_count as usize) <= dest_memory.memory.len(),
            "Destination range exceeds memory bounds"
        );

        unsafe {
            ptr::copy(src_ptr, dest_ptr, byte_count as usize);
        }
    }

    pub fn fill(&mut self, offset: i32, byte_count: i32, value: u8) {
        let start = offset as usize;
        let end = start + byte_count as usize;

        debug_assert!(end <= self.memory.len(), "Fill range exceeds memory bounds");

        self.memory[start..end].fill(value);
    }

    pub fn find_null(&self, address: i32) -> i32 {
        let mut offset: usize = address as usize;
        let len: usize = self.memory.len();

        while offset + VECTOR_SIZE <= len {
            let chunk = unsafe {
                let i8_slice = slice::from_raw_parts(
                    self.memory.as_ptr().add(offset) as *const i8,
                    VECTOR_SIZE,
                );
                Simd::<i8, VECTOR_SIZE>::from_slice(i8_slice)
            };
            let mask = chunk.simd_eq(Simd::splat(0));

            if mask.any() {
                let first_null: usize = mask.to_bitmask().trailing_zeros() as usize;
                return (offset + first_null) as i32;
            }

            offset += VECTOR_SIZE;
        }

        while offset < len {
            if self.memory[offset] == 0 {
                return offset as i32;
            }
            offset += 1;
        }

        -1
    }

    pub unsafe fn read_i32(&self, address: i32) -> i32 {
        let pointer = self.memory.as_ptr().add(address as usize) as *const i32;
        ptr::read_unaligned(pointer)
    }

    pub fn read_i32_from_i8(&self, address: i32) -> i32 {
        let pointer = address as usize;
        let bytes = &self.memory[pointer..pointer + 1];
        i8::from_le_bytes(bytes.try_into().unwrap()) as i32
    }

    pub fn read_i32_from_i16(&self, address: i32) -> i32 {
        let pointer = address as usize;
        let bytes = &self.memory[pointer..pointer + 2];
        i16::from_le_bytes(bytes.try_into().unwrap()) as i32
    }

    pub fn read_i32_from_u8(&self, address: i32) -> i32 {
        let pointer = address as usize;
        let bytes = &self.memory[pointer..pointer + 1];
        u8::from_le_bytes(bytes.try_into().unwrap()) as i32
    }

    pub fn read_i32_from_u16(&self, address: i32) -> i32 {
        let pointer = address as usize;
        let bytes = &self.memory[pointer..pointer + 2];
        u16::from_le_bytes(bytes.try_into().unwrap()) as i32
    }

    pub fn write_i32(&mut self, address: i32, value: i32) {
        let pointer = address as usize;
        self.memory[pointer..pointer + 4].copy_from_slice(&value.to_le_bytes());
    }

    pub fn write_i32_to_i8(&mut self, address: i32, value: i32) {
        let pointer = address as usize;
        let bytes = (value as i8).to_le_bytes();
        self.memory[pointer..pointer + 1].copy_from_slice(&bytes);
    }

    pub fn write_i32_to_i16(&mut self, address: i32, value: i32) {
        let pointer = address as usize;
        let bytes = (value as i16).to_le_bytes();
        self.memory[pointer..pointer + 2].copy_from_slice(&bytes);
    }

    pub fn read_i64(&self, address: i32) -> i64 {
        let pointer = address as usize;
        let bytes = &self.memory[pointer..pointer + 8];
        i64::from_le_bytes(bytes.try_into().unwrap())
    }

    pub fn read_i64_from_i8(&self, address: i32) -> i64 {
        let pointer = address as usize;
        let bytes = &self.memory[pointer..pointer + 1];
        i8::from_le_bytes(bytes.try_into().unwrap()) as i64
    }

    pub fn read_i64_from_i16(&self, address: i32) -> i64 {
        let pointer = address as usize;
        let bytes = &self.memory[pointer..pointer + 2];
        i16::from_le_bytes(bytes.try_into().unwrap()) as i64
    }

    pub fn read_i64_from_i32(&self, address: i32) -> i64 {
        let pointer = address as usize;
        let bytes = &self.memory[pointer..pointer + 4];
        i32::from_le_bytes(bytes.try_into().unwrap()) as i64
    }

    pub fn read_i64_from_u8(&self, address: i32) -> i64 {
        let pointer = address as usize;
        let bytes = &self.memory[pointer..pointer + 1];
        u8::from_le_bytes(bytes.try_into().unwrap()) as i64
    }

    pub fn read_i64_from_u16(&self, address: i32) -> i64 {
        let pointer = address as usize;
        let bytes = &self.memory[pointer..pointer + 2];
        u16::from_le_bytes(bytes.try_into().unwrap()) as i64
    }

    pub fn read_i64_from_u32(&self, address: i32) -> i64 {
        let pointer = address as usize;
        let bytes = &self.memory[pointer..pointer + 4];
        u32::from_le_bytes(bytes.try_into().unwrap()) as i64
    }

    pub fn write_i64(&mut self, address: i32, value: i64) {
        let pointer = address as usize;
        self.memory[pointer..pointer + 8].copy_from_slice(&value.to_le_bytes());
    }

    pub fn write_i64_to_i8(&mut self, address: i32, value: i64) {
        let pointer = address as usize;
        let bytes = (value as i8).to_le_bytes();
        self.memory[pointer..pointer + 1].copy_from_slice(&bytes);
    }

    pub fn write_i64_to_i16(&mut self, address: i32, value: i64) {
        let pointer = address as usize;
        let bytes = (value as i16).to_le_bytes();
        self.memory[pointer..pointer + 2].copy_from_slice(&bytes);
    }

    pub fn write_i64_to_i32(&mut self, address: i32, value: i64) {
        let pointer = address as usize;
        let bytes = (value as i32).to_le_bytes();
        self.memory[pointer..pointer + 4].copy_from_slice(&bytes);
    }

    pub fn read_f32(&self, address: i32) -> f32 {
        let pointer = address as usize;
        let bytes = &self.memory[pointer..pointer + 4];
        f32::from_le_bytes(bytes.try_into().unwrap())
    }

    pub fn write_f32(&mut self, address: i32, value: f32) {
        let pointer = address as usize;
        self.memory[pointer..pointer + 4].copy_from_slice(&value.to_le_bytes());
    }

    pub fn read_f64(&self, address: i32) -> f64 {
        let pointer = address as usize;
        let bytes = &self.memory[pointer..pointer + 8];
        f64::from_le_bytes(bytes.try_into().unwrap())
    }

    pub fn write_f64(&mut self, address: i32, value: f64) {
        let pointer = address as usize;
        self.memory[pointer..pointer + 8].copy_from_slice(&value.to_le_bytes());
    }

    pub fn read_bytes(&self, address: i32, byte_count: usize) -> &[u8] {
        let start = address as usize;
        let end = start + byte_count;

        debug_assert!(end <= self.memory.len(), "Read range exceeds memory bounds");

        &self.memory[start..end]
    }

    pub fn write_bytes(&mut self, address: i32, bytearray: &[u8]) {
        let start = address as usize;
        let end = start + bytearray.len();

        debug_assert!(
            end <= self.memory.len(),
            "The provided byte array exceeds the memory bounds"
        );

        self.memory[start..end].copy_from_slice(bytearray);
    }

    pub fn atomic_read_i32(&self, address: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI32;
        unsafe { (*aligned_ptr).load(Ordering::SeqCst) }
    }

    pub fn atomic_read_i32_from_i8(&self, address: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI8;
        unsafe { (*aligned_ptr).load(Ordering::SeqCst) as i32 }
    }

    pub fn atomic_read_i32_from_i16(&self, address: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI16;
        unsafe { (*aligned_ptr).load(Ordering::SeqCst) as i32 }
    }

    pub fn atomic_read_i32_from_u8(&self, address: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicU8;
        unsafe { (*aligned_ptr).load(Ordering::SeqCst) as i32 }
    }

    pub fn atomic_read_i32_from_u16(&self, address: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicU16;
        unsafe { (*aligned_ptr).load(Ordering::SeqCst) as i32 }
    }

    pub fn atomic_write_i32(&mut self, address: i32, value: i32) {
        let aligned_ptr = self.memory[address as usize..].as_mut_ptr() as *mut AtomicI32;
        unsafe { (*aligned_ptr).store(value, Ordering::SeqCst) }
    }

    pub fn atomic_write_i32_to_i8(&mut self, address: i32, value: i32) {
        let aligned_ptr = self.memory[address as usize..].as_mut_ptr() as *mut AtomicI8;
        unsafe { (*aligned_ptr).store(value as i8, Ordering::SeqCst) }
    }

    pub fn atomic_write_i32_to_i16(&mut self, address: i32, value: i32) {
        let aligned_ptr = self.memory[address as usize..].as_mut_ptr() as *mut AtomicI16;
        unsafe { (*aligned_ptr).store(value as i16, Ordering::SeqCst) }
    }

    pub fn atomic_read_i64(&self, address: i32) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI64;
        unsafe { (*aligned_ptr).load(Ordering::SeqCst) }
    }

    pub fn atomic_read_i64_from_i8(&self, address: i32) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI8;
        unsafe { (*aligned_ptr).load(Ordering::SeqCst) as i64 }
    }

    pub fn atomic_read_i64_from_i16(&self, address: i32) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI16;
        unsafe { (*aligned_ptr).load(Ordering::SeqCst) as i64 }
    }

    pub fn atomic_read_i64_from_i32(&self, address: i32) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI32;
        unsafe { (*aligned_ptr).load(Ordering::SeqCst) as i64 }
    }

    pub fn atomic_read_i64_from_u8(&self, address: i32) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicU8;
        unsafe { (*aligned_ptr).load(Ordering::SeqCst) as i64 }
    }

    pub fn atomic_read_i64_from_u16(&self, address: i32) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicU16;
        unsafe { (*aligned_ptr).load(Ordering::SeqCst) as i64 }
    }

    pub fn atomic_read_i64_from_u32(&self, address: i32) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicU32;
        unsafe { (*aligned_ptr).load(Ordering::SeqCst) as i64 }
    }

    pub fn atomic_write_i64(&mut self, address: i32, value: i64) {
        let aligned_ptr = self.memory[address as usize..].as_mut_ptr() as *mut AtomicI64;
        unsafe { (*aligned_ptr).store(value, Ordering::SeqCst) }
    }

    pub fn atomic_write_i64_to_i8(&mut self, address: i32, value: i64) {
        let aligned_ptr = self.memory[address as usize..].as_mut_ptr() as *mut AtomicI8;
        unsafe { (*aligned_ptr).store(value as i8, Ordering::SeqCst) }
    }

    pub fn atomic_write_i64_to_i16(&mut self, address: i32, value: i64) {
        let aligned_ptr = self.memory[address as usize..].as_mut_ptr() as *mut AtomicI16;
        unsafe { (*aligned_ptr).store(value as i16, Ordering::SeqCst) }
    }

    pub fn atomic_write_i64_to_i32(&mut self, address: i32, value: i64) {
        let aligned_ptr = self.memory[address as usize..].as_mut_ptr() as *mut AtomicI32;
        unsafe { (*aligned_ptr).store(value as i32, Ordering::SeqCst) }
    }

    pub fn atomic_rmw_add_i32(&self, address: i32, value: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI32;
        unsafe { (*aligned_ptr).fetch_add(value, Ordering::SeqCst) }
    }

    pub fn atomic_rmw_and_i32(&self, address: i32, value: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI32;
        unsafe { (*aligned_ptr).fetch_and(value, Ordering::SeqCst) }
    }

    pub fn atomic_rmw_sub_i32(&self, address: i32, value: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI32;
        unsafe { (*aligned_ptr).fetch_sub(value, Ordering::SeqCst) }
    }

    pub fn atomic_rmw_or_i32(&self, address: i32, value: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI32;
        unsafe { (*aligned_ptr).fetch_or(value, Ordering::SeqCst) }
    }

    pub fn atomic_rmw_xor_i32(&self, address: i32, value: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI32;
        unsafe { (*aligned_ptr).fetch_xor(value, Ordering::SeqCst) }
    }

    pub fn atomic_rmw_exchange_i32(&self, address: i32, value: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI32;
        unsafe { (*aligned_ptr).swap(value, Ordering::SeqCst) }
    }

    pub fn atomic_rmw_add_i32_to_i8(&self, address: i32, value: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI8;
        unsafe { (*aligned_ptr).fetch_add(value as i8, Ordering::SeqCst) as i32 }
    }

    pub fn atomic_rmw_and_i32_to_i8(&self, address: i32, value: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI8;
        unsafe { (*aligned_ptr).fetch_and(value as i8, Ordering::SeqCst) as i32 }
    }

    pub fn atomic_rmw_sub_i32_to_i8(&self, address: i32, value: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI8;
        unsafe { (*aligned_ptr).fetch_sub(value as i8, Ordering::SeqCst) as i32 }
    }

    pub fn atomic_rmw_or_i32_to_i8(&self, address: i32, value: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI8;
        unsafe { (*aligned_ptr).fetch_or(value as i8, Ordering::SeqCst) as i32 }
    }

    pub fn atomic_rmw_xor_i32_to_i8(&self, address: i32, value: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI8;
        unsafe { (*aligned_ptr).fetch_xor(value as i8, Ordering::SeqCst) as i32 }
    }

    pub fn atomic_rmw_exchange_i32_to_i8(&self, address: i32, value: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI8;
        unsafe { (*aligned_ptr).swap(value as i8, Ordering::SeqCst) as i32 }
    }

    pub fn atomic_rmw_add_i32_to_i16(&self, address: i32, value: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI16;
        unsafe { (*aligned_ptr).fetch_add(value as i16, Ordering::SeqCst) as i32 }
    }

    pub fn atomic_rmw_and_i32_to_i16(&self, address: i32, value: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI16;
        unsafe { (*aligned_ptr).fetch_and(value as i16, Ordering::SeqCst) as i32 }
    }

    pub fn atomic_rmw_sub_i32_to_i16(&self, address: i32, value: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI16;
        unsafe { (*aligned_ptr).fetch_sub(value as i16, Ordering::SeqCst) as i32 }
    }

    pub fn atomic_rmw_or_i32_to_i16(&self, address: i32, value: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI16;
        unsafe { (*aligned_ptr).fetch_or(value as i16, Ordering::SeqCst) as i32 }
    }

    pub fn atomic_rmw_xor_i32_to_i16(&self, address: i32, value: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI16;
        unsafe { (*aligned_ptr).fetch_xor(value as i16, Ordering::SeqCst) as i32 }
    }

    pub fn atomic_rmw_exchange_i32_to_i16(&self, address: i32, value: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI16;
        unsafe { (*aligned_ptr).swap(value as i16, Ordering::SeqCst) as i32 }
    }

    pub fn atomic_rmw_add_i64(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI64;
        unsafe { (*aligned_ptr).fetch_add(value, Ordering::SeqCst) }
    }

    pub fn atomic_rmw_and_i64(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI64;
        unsafe { (*aligned_ptr).fetch_and(value, Ordering::SeqCst) }
    }

    pub fn atomic_rmw_sub_i64(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI64;
        unsafe { (*aligned_ptr).fetch_sub(value, Ordering::SeqCst) }
    }

    pub fn atomic_rmw_or_i64(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI64;
        unsafe { (*aligned_ptr).fetch_or(value, Ordering::SeqCst) }
    }

    pub fn atomic_rmw_xor_i64(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI64;
        unsafe { (*aligned_ptr).fetch_xor(value, Ordering::SeqCst) }
    }

    pub fn atomic_rmw_exchange_i64(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI64;
        unsafe { (*aligned_ptr).swap(value, Ordering::SeqCst) }
    }

    pub fn atomic_rmw_add_i64_to_i8(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI8;
        unsafe { (*aligned_ptr).fetch_add(value as i8, Ordering::SeqCst) as i64 }
    }

    pub fn atomic_rmw_and_i64_to_i8(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI8;
        unsafe { (*aligned_ptr).fetch_and(value as i8, Ordering::SeqCst) as i64 }
    }

    pub fn atomic_rmw_sub_i64_to_i8(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI8;
        unsafe { (*aligned_ptr).fetch_sub(value as i8, Ordering::SeqCst) as i64 }
    }

    pub fn atomic_rmw_or_i64_to_i8(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI8;
        unsafe { (*aligned_ptr).fetch_or(value as i8, Ordering::SeqCst) as i64 }
    }

    pub fn atomic_rmw_xor_i64_to_i8(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI8;
        unsafe { (*aligned_ptr).fetch_xor(value as i8, Ordering::SeqCst) as i64 }
    }

    pub fn atomic_rmw_exchange_i64_to_i8(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI8;
        unsafe { (*aligned_ptr).swap(value as i8, Ordering::SeqCst) as i64 }
    }

    pub fn atomic_rmw_add_i64_to_i16(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI16;
        unsafe { (*aligned_ptr).fetch_add(value as i16, Ordering::SeqCst) as i64 }
    }

    pub fn atomic_rmw_and_i64_to_i16(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI16;
        unsafe { (*aligned_ptr).fetch_and(value as i16, Ordering::SeqCst) as i64 }
    }

    pub fn atomic_rmw_sub_i64_to_i16(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI16;
        unsafe { (*aligned_ptr).fetch_sub(value as i16, Ordering::SeqCst) as i64 }
    }

    pub fn atomic_rmw_or_i64_to_i16(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI16;
        unsafe { (*aligned_ptr).fetch_or(value as i16, Ordering::SeqCst) as i64 }
    }

    pub fn atomic_rmw_xor_i64_to_i16(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI16;
        unsafe { (*aligned_ptr).fetch_xor(value as i16, Ordering::SeqCst) as i64 }
    }

    pub fn atomic_rmw_exchange_i64_to_i16(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI16;
        unsafe { (*aligned_ptr).swap(value as i16, Ordering::SeqCst) as i64 }
    }

    pub fn atomic_rmw_add_i64_to_i32(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI32;
        unsafe { (*aligned_ptr).fetch_add(value as i32, Ordering::SeqCst) as i64 }
    }

    pub fn atomic_rmw_and_i64_to_i32(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI32;
        unsafe { (*aligned_ptr).fetch_and(value as i32, Ordering::SeqCst) as i64 }
    }

    pub fn atomic_rmw_sub_i64_to_i32(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI32;
        unsafe { (*aligned_ptr).fetch_sub(value as i32, Ordering::SeqCst) as i64 }
    }

    pub fn atomic_rmw_or_i64_to_i32(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI32;
        unsafe { (*aligned_ptr).fetch_or(value as i32, Ordering::SeqCst) as i64 }
    }

    pub fn atomic_rmw_xor_i64_to_i32(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI32;
        unsafe { (*aligned_ptr).fetch_xor(value as i32, Ordering::SeqCst) as i64 }
    }

    pub fn atomic_rmw_exchange_i64_to_i32(&self, address: i32, value: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI32;
        unsafe { (*aligned_ptr).swap(value as i32, Ordering::SeqCst) as i64 }
    }

    pub fn atomic_compare_exchange_i32(&self, address: i32, current: i32, new: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI32;
        unsafe {
            (*aligned_ptr)
                .compare_exchange(current, new, Ordering::SeqCst, Ordering::SeqCst)
                .unwrap_or_else(|value| value)
        }
    }

    pub fn atomic_compare_exchange_i32_to_i8(&self, address: i32, current: i32, new: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI8;
        unsafe {
            match (*aligned_ptr).compare_exchange(
                current as i8,
                new as i8,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(value) => value as i32,
                Err(value) => value as i32,
            }
        }
    }

    pub fn atomic_compare_exchange_i32_to_i16(&self, address: i32, current: i32, new: i32) -> i32 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI16;
        unsafe {
            match (*aligned_ptr).compare_exchange(
                current as i16,
                new as i16,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(value) => value as i32,
                Err(value) => value as i32,
            }
        }
    }

    pub fn atomic_compare_exchange_i64(&self, address: i32, current: i64, new: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI64;
        unsafe {
            (*aligned_ptr)
                .compare_exchange(current, new, Ordering::SeqCst, Ordering::SeqCst)
                .unwrap_or_else(|value| value)
        }
    }

    pub fn atomic_compare_exchange_i64_to_i8(&self, address: i32, current: i64, new: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI8;
        unsafe {
            match (*aligned_ptr).compare_exchange(
                current as i8,
                new as i8,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(value) => value as i64,
                Err(value) => value as i64,
            }
        }
    }

    pub fn atomic_compare_exchange_i64_to_i16(&self, address: i32, current: i64, new: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI16;
        unsafe {
            match (*aligned_ptr).compare_exchange(
                current as i16,
                new as i16,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(value) => value as i64,
                Err(value) => value as i64,
            }
        }
    }

    pub fn atomic_compare_exchange_i64_to_i32(&self, address: i32, current: i64, new: i64) -> i64 {
        let aligned_ptr = self.memory[address as usize..].as_ptr() as *const AtomicI32;
        unsafe {
            match (*aligned_ptr).compare_exchange(
                current as i32,
                new as i32,
                Ordering::SeqCst,
                Ordering::SeqCst,
            ) {
                Ok(value) => value as i64,
                Err(value) => value as i64,
            }
        }
    }

    pub fn atomic_fence(&self) {
        std::sync::atomic::fence(Ordering::SeqCst);
    }

    #[inline(always)]
    fn wait(&self, addr: i32, timeout_nanos: i64) -> i32 {
        let wait_entry = Arc::new(WaitEntry {
            condvar: Condvar::new(),
            mutex: Mutex::new(()),
        });

        let mut guard = wait_entry.mutex.lock();
        let queue = self
            .wait_queues
            .entry(addr)
            .or_insert_with(|| Arc::new(ConcurrentQueue::unbounded()))
            .clone();

        queue.push(wait_entry.clone()).unwrap();

        let timed_out = if timeout_nanos >= 0 {
            let timeout = Duration::from_nanos(timeout_nanos as u64);
            let instant = Instant::now() + timeout;
            wait_entry
                .condvar
                .wait_until(&mut guard, instant)
                .timed_out()
        } else {
            wait_entry.condvar.wait(&mut guard);
            false
        };

        if timed_out {
            2
        } else {
            0
        }
    }

    pub fn wait_i32(&self, addr: i32, expected: i32, timeout_nanos: i64) -> i32 {
        let atomic = unsafe { &*(self.memory.as_ptr().add(addr as usize) as *const AtomicI32) };

        if atomic.load(Ordering::SeqCst) != expected {
            return 1;
        }

        self.wait(addr, timeout_nanos)
    }

    pub fn wait_i64(&self, addr: i32, expected: i64, timeout_nanos: i64) -> i32 {
        let atomic = unsafe { &*(self.memory.as_ptr().add(addr as usize) as *const AtomicI64) };

        if atomic.load(Ordering::SeqCst) != expected {
            return 1;
        }

        self.wait(addr, timeout_nanos)
    }

    pub fn notify(&self, addr: i32, count: i32) -> i32 {
        let mut woken_count = 0;

        let Some(queue) = self.wait_queues.get(&addr) else {
            return woken_count;
        };

        for _ in 0..count {
            if let Ok(entry) = queue.pop() {
                let guard = entry.mutex.lock();
                let woken = entry.condvar.notify_one();
                woken_count += woken as i32;
                drop(guard);
            } else {
                break;
            }
        }

        woken_count
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{Arc, Barrier};
    use std::thread;

    #[test]
    fn test_grow() {
        const INITIAL_PAGES: u32 = 2;
        const GROW_PAGES: u32 = 3;

        let mut memory = LinearMemory::new(INITIAL_PAGES);

        let initial_size = (INITIAL_PAGES * PAGE_SIZE) as usize;
        memory.memory[0] = 42;
        memory.memory[initial_size - 1] = 99;

        assert!(memory.grow(GROW_PAGES));

        let new_size = memory.memory.len();
        let expected_size = (INITIAL_PAGES + GROW_PAGES) * PAGE_SIZE;
        assert_eq!(new_size, expected_size as usize);

        assert_eq!(memory.memory[0], 42);
        assert_eq!(memory.memory[initial_size - 1], 99);

        assert_eq!(memory.memory[initial_size], 0);
        assert_eq!(memory.memory[new_size - 1], 0);
    }

    #[test]
    fn test_copy() {
        let mut src_memory = LinearMemory::new(1);
        let mut dest_memory = LinearMemory::new(1);

        let src_offset = 0;
        let dest_offset = 4;
        let byte_count = 4;

        src_memory.memory[src_offset as usize..(src_offset + byte_count) as usize]
            .copy_from_slice(&[1, 2, 3, 4]);

        src_memory.copy(src_offset, &mut dest_memory, dest_offset, byte_count);

        assert_eq!(
            &dest_memory.memory[dest_offset as usize..(dest_offset + byte_count) as usize],
            &[1, 2, 3, 4]
        );
    }

    #[test]
    fn test_fill() {
        let mut linear_memory = LinearMemory::new(1);

        let offset = 32;
        let byte_count = 16;
        let value = 0xFF;

        linear_memory.fill(offset, byte_count, value);

        assert_eq!(
            &linear_memory.memory[offset as usize..(offset + byte_count) as usize],
            &[value; 16]
        );
    }

    #[test]
    fn test_find_null() {
        let mut memory = LinearMemory::new(1);

        let offset = 64;
        let bytes = [1, 2, 3, 4, 5, 6, 7];
        let null_byte = 0;

        memory.memory[offset..offset + bytes.len()].copy_from_slice(&bytes);
        memory.memory[offset + bytes.len()] = null_byte;

        let null_offset = memory.find_null(offset as i32);

        assert_eq!(null_offset, (offset + bytes.len()) as i32);
    }

    #[test]
    fn test_i32_rw() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let value = 117;

        memory.write_i32(address, value);

        let read_value = unsafe { memory.read_i32(address) };
        assert_eq!(value, read_value);
    }

    #[test]
    fn test_i32_rw_i8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let value: i8 = -128;

        memory.write_i32_to_i8(address, value as i32);

        let read_value = memory.read_i32_from_i8(address);
        assert_eq!(value as i32, read_value);
    }

    #[test]
    fn test_i32_rw_i16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let value: i16 = -32768;

        memory.write_i32_to_i16(address, value as i32);

        let read_value = memory.read_i32_from_i16(address);
        assert_eq!(value as i32, read_value);
    }

    #[test]
    fn test_i32_rw_u8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let value: u8 = 255;

        memory.write_i32_to_i8(address, value as i32);

        let read_value = memory.read_i32_from_u8(address);
        assert_eq!(value as i32, read_value);
    }

    #[test]
    fn test_i32_rw_u16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let value: u16 = 65535;

        memory.write_i32_to_i16(address, value as i32);

        let read_value = memory.read_i32_from_u16(address);
        assert_eq!(value as i32, read_value);
    }

    #[test]
    fn test_i64_rw() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 64;
        let value = 12345678910;

        memory.write_i64(address, value);

        let read_value = memory.read_i64(address);
        assert_eq!(value, read_value);
    }

    #[test]
    fn test_i64_rw_i8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let value: i8 = -128;

        memory.write_i64_to_i8(address, value as i64);

        let read_value = memory.read_i64_from_i8(address);
        assert_eq!(value as i64, read_value);
    }

    #[test]
    fn test_i64_rw_i16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let value: i16 = -32768;

        memory.write_i64_to_i16(address, value as i64);

        let read_value = memory.read_i64_from_i16(address);
        assert_eq!(value as i64, read_value);
    }

    #[test]
    fn test_i64_rw_i32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let value: i32 = -2147483648;

        memory.write_i64_to_i32(address, value as i64);

        let read_value = memory.read_i64_from_i32(address);
        assert_eq!(value as i64, read_value);
    }

    #[test]
    fn test_i64_rw_u8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let value: u8 = 255;

        memory.write_i64_to_i8(address, value as i64);

        let read_value = memory.read_i64_from_u8(address);
        assert_eq!(value as i64, read_value);
    }

    #[test]
    fn test_i64_rw_u16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let value: u16 = 65535;

        memory.write_i64_to_i16(address, value as i64);

        let read_value = memory.read_i64_from_u16(address);
        assert_eq!(value as i64, read_value);
    }

    #[test]
    fn test_i64_rw_u32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let value: u32 = 4294967295;

        memory.write_i64(address, value as i64);

        let read_value = memory.read_i64_from_u32(address);
        assert_eq!(value as i64, read_value);
    }

    #[test]
    fn test_f32_rw() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let value = 123.456_f32;

        memory.write_f32(address, value);

        let read_value = memory.read_f32(address);
        assert!((value - read_value).abs() < f32::EPSILON);
    }

    #[test]
    fn test_f64_rw() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 64;
        let value = 98765.4321_f64;

        memory.write_f64(address, value);

        let read_value = memory.read_f64(address);
        assert!((value - read_value).abs() < f64::EPSILON);
    }

    #[test]
    fn test_f32_rw_edge_cases() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 128;

        let value = f32::INFINITY;
        memory.write_f32(address, value);
        let read_value = memory.read_f32(address);
        assert!(read_value.is_infinite() && read_value.is_sign_positive());

        let value = f32::NEG_INFINITY;
        memory.write_f32(address, value);
        let read_value = memory.read_f32(address);
        assert!(read_value.is_infinite() && read_value.is_sign_negative());

        let value = f32::NAN;
        memory.write_f32(address, value);
        let read_value = memory.read_f32(address);
        assert!(read_value.is_nan());
    }

    #[test]
    fn test_f64_rw_edge_cases() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 256;

        let value = f64::INFINITY;
        memory.write_f64(address, value);
        let read_value = memory.read_f64(address);
        assert!(read_value.is_infinite() && read_value.is_sign_positive());

        let value = f64::NEG_INFINITY;
        memory.write_f64(address, value);
        let read_value = memory.read_f64(address);
        assert!(read_value.is_infinite() && read_value.is_sign_negative());

        let value = f64::NAN;
        memory.write_f64(address, value);
        let read_value = memory.read_f64(address);
        assert!(read_value.is_nan());
    }

    #[test]
    fn test_read_bytes() {
        let mut linear_memory = LinearMemory::new(1);

        let test_data = [10, 20, 30, 40];
        let address = 64;
        linear_memory.write_bytes(address, &test_data);

        let read_data = linear_memory.read_bytes(address, test_data.len());

        assert_eq!(read_data, &test_data);
    }

    #[test]
    fn test_write_bytes() {
        let mut linear_memory = LinearMemory::new(1);

        let bytearray = [1, 2, 3, 4, 5];
        let address = 64;

        linear_memory.write_bytes(address, &bytearray);

        assert_eq!(
            &linear_memory.memory[address as usize..(address as usize + bytearray.len())],
            &bytearray
        );
    }

    #[test]
    fn test_atomic_read_i32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 4;
        let value: i32 = 117;

        memory.write_i32(address, value);

        let read_value = memory.atomic_read_i32(address);
        assert_eq!(value, read_value);
    }

    #[test]
    fn test_atomic_read_i32_from_i8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let value: i8 = -128;

        memory.write_i32_to_i8(address, value as i32);

        let read_value = memory.atomic_read_i32_from_i8(address);
        assert_eq!(value as i32, read_value);
    }

    #[test]
    fn test_atomic_read_i32_from_i16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 2;
        let value: i16 = -32768;

        memory.write_i32_to_i16(address, value as i32);

        let read_value = memory.atomic_read_i32_from_i16(address);
        assert_eq!(value as i32, read_value);
    }

    #[test]
    fn test_atomic_read_i32_from_u8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 1;
        let value: u8 = 255;

        memory.write_i32_to_i8(address, value as i32);

        let read_value = memory.atomic_read_i32_from_u8(address);
        assert_eq!(value as i32, read_value);
    }

    #[test]
    fn test_atomic_read_i32_from_u16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 4;
        let value: u16 = 65535;

        memory.write_i32_to_i16(address, value as i32);

        let read_value = memory.atomic_read_i32_from_u16(address);
        assert_eq!(value as i32, read_value);
    }

    #[test]
    fn test_atomic_write_i32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 4;
        let value: i32 = 117;

        memory.atomic_write_i32(address, value);

        let read_value = unsafe { memory.read_i32(address) };
        assert_eq!(value, read_value);
    }

    #[test]
    fn test_atomic_write_i32_to_i8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let value: i32 = -128;

        memory.atomic_write_i32_to_i8(address, value);

        let read_value = memory.read_i32_from_i8(address);
        assert_eq!(value as i8 as i32, read_value);
    }

    #[test]
    fn test_atomic_write_i32_to_i16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 2;
        let value: i32 = -32768;

        memory.atomic_write_i32_to_i16(address, value);

        let read_value = memory.read_i32_from_i16(address);
        assert_eq!(value as i16 as i32, read_value);
    }

    #[test]
    fn test_atomic_read_i64() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 8;
        let value: i64 = 123456789101112;

        memory.write_i64(address, value);

        let read_value = memory.atomic_read_i64(address);
        assert_eq!(value, read_value);
    }

    #[test]
    fn test_atomic_read_i64_from_i8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let value: i8 = -128;

        memory.write_i32_to_i8(address, value as i32);

        let read_value = memory.atomic_read_i64_from_i8(address);
        assert_eq!(value as i64, read_value);
    }

    #[test]
    fn test_atomic_read_i64_from_i16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 2;
        let value: i16 = -32768;

        memory.write_i32_to_i16(address, value as i32);

        let read_value = memory.atomic_read_i64_from_i16(address);
        assert_eq!(value as i64, read_value);
    }

    #[test]
    fn test_atomic_read_i64_from_i32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 4;
        let value: i32 = -2147483648;

        memory.write_i32(address, value);

        let read_value = memory.atomic_read_i64_from_i32(address);
        assert_eq!(value as i64, read_value);
    }

    #[test]
    fn test_atomic_read_i64_from_u8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 1;
        let value: u8 = 255;

        memory.write_i32_to_i8(address, value as i32);

        let read_value = memory.atomic_read_i64_from_u8(address);
        assert_eq!(value as i64, read_value);
    }

    #[test]
    fn test_atomic_read_i64_from_u16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 6;
        let value: u16 = 65535;

        memory.write_i32_to_i16(address, value as i32);

        let read_value = memory.atomic_read_i64_from_u16(address);
        assert_eq!(value as i64, read_value);
    }

    #[test]
    fn test_atomic_read_i64_from_u32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 8;
        let value: u32 = 4294967295;

        memory.write_i32(address, value as i32);

        let read_value = memory.atomic_read_i64_from_u32(address);
        assert_eq!(value as i64, read_value);
    }

    #[test]
    fn test_atomic_write_i64() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 8;
        let value: i64 = 123456789101112;

        memory.atomic_write_i64(address, value);

        let read_value = memory.read_i64(address);
        assert_eq!(value, read_value);
    }

    #[test]
    fn test_atomic_write_i64_to_i8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let value: i64 = -128;

        memory.atomic_write_i64_to_i8(address, value);

        let read_value = memory.read_i64_from_i8(address);
        assert_eq!(value as i8 as i64, read_value);
    }

    #[test]
    fn test_atomic_write_i64_to_i16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 2;
        let value: i64 = -32768;

        memory.atomic_write_i64_to_i16(address, value);

        let read_value = memory.read_i64_from_i16(address);
        assert_eq!(value as i16 as i64, read_value);
    }

    #[test]
    fn test_atomic_write_i64_to_i32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 4;
        let value: i64 = -2147483648;

        memory.atomic_write_i64_to_i32(address, value);

        let read_value = memory.read_i64_from_i32(address);
        assert_eq!(value as i32 as i64, read_value);
    }

    #[test]
    fn test_atomic_rmw_add_i32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let initial_value: i32 = 42;
        let add_value: i32 = 58;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI32;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_add_i32(address, add_value);

        assert_eq!(previous_value, initial_value);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value + add_value);
    }

    #[test]
    fn test_atomic_rmw_and_i32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 4;
        let initial_value: i32 = 0b1100;
        let and_value: i32 = 0b1010;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI32;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_and_i32(address, and_value);

        assert_eq!(previous_value, initial_value);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value & and_value);
    }

    #[test]
    fn test_atomic_rmw_sub_i32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 8;
        let initial_value: i32 = 100;
        let sub_value: i32 = 30;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI32;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_sub_i32(address, sub_value);

        assert_eq!(previous_value, initial_value);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value - sub_value);
    }

    #[test]
    fn test_atomic_rmw_or_i32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 12;
        let initial_value: i32 = 0b1100;
        let or_value: i32 = 0b1010;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI32;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_or_i32(address, or_value);

        assert_eq!(previous_value, initial_value);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value | or_value);
    }

    #[test]
    fn test_atomic_rmw_xor_i32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 16;
        let initial_value: i32 = 0b1100;
        let xor_value: i32 = 0b1010;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI32;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_xor_i32(address, xor_value);

        assert_eq!(previous_value, initial_value);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value ^ xor_value);
    }

    #[test]
    fn test_atomic_rmw_exchange_i32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 20;
        let initial_value: i32 = 12345;
        let exchange_value: i32 = 54321;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI32;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_exchange_i32(address, exchange_value);

        assert_eq!(previous_value, initial_value);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, exchange_value);
    }

    #[test]
    fn test_atomic_rmw_add_i32_to_i8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let initial_value: i8 = 42;
        let add_value: i8 = 58;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI8;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_add_i32_to_i8(address, add_value as i32);

        assert_eq!(previous_value, initial_value as i32);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value + add_value);
    }

    #[test]
    fn test_atomic_rmw_and_i32_to_i8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 4;
        let initial_value: i8 = 0b1100;
        let and_value: i8 = 0b1010;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI8;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_and_i32_to_i8(address, and_value as i32);

        assert_eq!(previous_value, initial_value as i32);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value & and_value);
    }

    #[test]
    fn test_atomic_rmw_sub_i32_to_i8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 8;
        let initial_value: i8 = 100;
        let sub_value: i8 = 30;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI8;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_sub_i32_to_i8(address, sub_value as i32);

        assert_eq!(previous_value, initial_value as i32);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value - sub_value);
    }

    #[test]
    fn test_atomic_rmw_or_i32_to_i8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 12;
        let initial_value: i8 = 0b1100;
        let or_value: i8 = 0b1010;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI8;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_or_i32_to_i8(address, or_value as i32);

        assert_eq!(previous_value, initial_value as i32);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value | or_value);
    }

    #[test]
    fn test_atomic_rmw_xor_i32_to_i8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 16;
        let initial_value: i8 = 0b1100;
        let xor_value: i8 = 0b1010;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI8;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_xor_i32_to_i8(address, xor_value as i32);

        assert_eq!(previous_value, initial_value as i32);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value ^ xor_value);
    }

    #[test]
    fn test_atomic_rmw_exchange_i32_to_i8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 20;
        let initial_value: i8 = 123;
        let exchange_value: i8 = 45;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI8;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_exchange_i32_to_i8(address, exchange_value as i32);

        assert_eq!(previous_value, initial_value as i32);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, exchange_value);
    }

    #[test]
    fn test_atomic_rmw_add_i32_to_i16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let initial_value: i16 = 1000;
        let add_value: i16 = 500;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI16;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_add_i32_to_i16(address, add_value as i32);

        assert_eq!(previous_value, initial_value as i32);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value + add_value);
    }

    #[test]
    fn test_atomic_rmw_and_i32_to_i16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 4;
        let initial_value: i16 = 0b1100_1100_1100;
        let and_value: i16 = 0b1010_1010_1010;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI16;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_and_i32_to_i16(address, and_value as i32);

        assert_eq!(previous_value, initial_value as i32);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value & and_value);
    }

    #[test]
    fn test_atomic_rmw_sub_i32_to_i16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 8;
        let initial_value: i16 = 3000;
        let sub_value: i16 = 1000;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI16;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_sub_i32_to_i16(address, sub_value as i32);

        assert_eq!(previous_value, initial_value as i32);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value - sub_value);
    }

    #[test]
    fn test_atomic_rmw_or_i32_to_i16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 12;
        let initial_value: i16 = 0b1100_1100_1100;
        let or_value: i16 = 0b1010_1010_1010;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI16;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_or_i32_to_i16(address, or_value as i32);

        assert_eq!(previous_value, initial_value as i32);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value | or_value);
    }

    #[test]
    fn test_atomic_rmw_xor_i32_to_i16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 16;
        let initial_value: i16 = 0b1100_1100_1100;
        let xor_value: i16 = 0b1010_1010_1010;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI16;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_xor_i32_to_i16(address, xor_value as i32);

        assert_eq!(previous_value, initial_value as i32);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value ^ xor_value);
    }

    #[test]
    fn test_atomic_rmw_exchange_i32_to_i16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 20;
        let initial_value: i16 = 5000;
        let exchange_value: i16 = 1000;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI16;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_exchange_i32_to_i16(address, exchange_value as i32);

        assert_eq!(previous_value, initial_value as i32);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, exchange_value);
    }

    #[test]
    fn test_atomic_rmw_add_i64() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let initial_value: i64 = 123456789101112;
        let add_value: i64 = 987654321;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI64;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_add_i64(address, add_value);

        assert_eq!(previous_value, initial_value);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value + add_value);
    }

    #[test]
    fn test_atomic_rmw_and_i64() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 8;
        let initial_value: i64 = -71777214294589696;
        let and_value: i64 = 71777214294589695;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI64;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_and_i64(address, and_value);

        assert_eq!(previous_value, initial_value);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value & and_value);
    }

    #[test]
    fn test_atomic_rmw_sub_i64() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 16;
        let initial_value: i64 = 1_000_000_000_000_000;
        let sub_value: i64 = 123_456_789_101_112;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI64;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_sub_i64(address, sub_value);

        assert_eq!(previous_value, initial_value);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value - sub_value);
    }

    #[test]
    fn test_atomic_rmw_or_i64() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 24;
        let initial_value: i64 = -71777214294589696;
        let or_value: i64 = 71777214294589695;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI64;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_or_i64(address, or_value);

        assert_eq!(previous_value, initial_value);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value | or_value);
    }

    #[test]
    fn test_atomic_rmw_xor_i64() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 32;
        let initial_value: i64 = -71777214294589696;
        let xor_value: i64 = 71777214294589695;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI64;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_xor_i64(address, xor_value);

        assert_eq!(previous_value, initial_value);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value ^ xor_value);
    }

    #[test]
    fn test_atomic_rmw_exchange_i64() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 40;
        let initial_value: i64 = 123_456_789_101_112;
        let exchange_value: i64 = 987_654_321_987_654;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI64;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_exchange_i64(address, exchange_value);

        assert_eq!(previous_value, initial_value);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, exchange_value);
    }

    #[test]
    fn test_atomic_rmw_add_i64_to_i8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let initial_value: i8 = 50;
        let add_value: i8 = 25;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI8;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_add_i64_to_i8(address, add_value as i64);

        assert_eq!(previous_value, initial_value as i64);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value + add_value);
    }

    #[test]
    fn test_atomic_rmw_and_i64_to_i8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 8;
        let initial_value: i8 = 12;
        let and_value: i8 = 10;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI8;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_and_i64_to_i8(address, and_value as i64);

        assert_eq!(previous_value, initial_value as i64);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value & and_value);
    }

    #[test]
    fn test_atomic_rmw_sub_i64_to_i8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 16;
        let initial_value: i8 = 100;
        let sub_value: i8 = 30;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI8;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_sub_i64_to_i8(address, sub_value as i64);

        assert_eq!(previous_value, initial_value as i64);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value - sub_value);
    }

    #[test]
    fn test_atomic_rmw_or_i64_to_i8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 24;
        let initial_value: i8 = 12;
        let or_value: i8 = 10;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI8;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_or_i64_to_i8(address, or_value as i64);

        assert_eq!(previous_value, initial_value as i64);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value | or_value);
    }

    #[test]
    fn test_atomic_rmw_xor_i64_to_i8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 32;
        let initial_value: i8 = 12;
        let xor_value: i8 = 10;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI8;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_xor_i64_to_i8(address, xor_value as i64);

        assert_eq!(previous_value, initial_value as i64);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value ^ xor_value);
    }

    #[test]
    fn test_atomic_rmw_exchange_i64_to_i8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 40;
        let initial_value: i8 = 100;
        let exchange_value: i8 = 50;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI8;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_exchange_i64_to_i8(address, exchange_value as i64);

        assert_eq!(previous_value, initial_value as i64);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, exchange_value);
    }

    #[test]
    fn test_atomic_rmw_add_i64_to_i16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let initial_value: i16 = 500;
        let add_value: i16 = 200;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI16;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_add_i64_to_i16(address, add_value as i64);

        assert_eq!(previous_value, initial_value as i64);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value + add_value);
    }

    #[test]
    fn test_atomic_rmw_and_i64_to_i16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 8;
        let initial_value: i16 = 1024;
        let and_value: i16 = 768;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI16;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_and_i64_to_i16(address, and_value as i64);

        assert_eq!(previous_value, initial_value as i64);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value & and_value);
    }

    #[test]
    fn test_atomic_rmw_sub_i64_to_i16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 16;
        let initial_value: i16 = 3000;
        let sub_value: i16 = 1000;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI16;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_sub_i64_to_i16(address, sub_value as i64);

        assert_eq!(previous_value, initial_value as i64);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value - sub_value);
    }

    #[test]
    fn test_atomic_rmw_or_i64_to_i16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 24;
        let initial_value: i16 = 1024;
        let or_value: i16 = 256;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI16;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_or_i64_to_i16(address, or_value as i64);

        assert_eq!(previous_value, initial_value as i64);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value | or_value);
    }

    #[test]
    fn test_atomic_rmw_xor_i64_to_i16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 32;
        let initial_value: i16 = 512;
        let xor_value: i16 = 256;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI16;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_xor_i64_to_i16(address, xor_value as i64);

        assert_eq!(previous_value, initial_value as i64);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value ^ xor_value);
    }

    #[test]
    fn test_atomic_rmw_exchange_i64_to_i16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 40;
        let initial_value: i16 = 1500;
        let exchange_value: i16 = 1000;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI16;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_exchange_i64_to_i16(address, exchange_value as i64);

        assert_eq!(previous_value, initial_value as i64);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, exchange_value);
    }

    #[test]
    fn test_atomic_rmw_add_i64_to_i32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let initial_value: i32 = 10_000;
        let add_value: i32 = 5_000;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI32;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_add_i64_to_i32(address, add_value as i64);

        assert_eq!(previous_value, initial_value as i64);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value + add_value);
    }

    #[test]
    fn test_atomic_rmw_and_i64_to_i32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 8;
        let initial_value: i32 = 12_000;
        let and_value: i32 = 8_000;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI32;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_and_i64_to_i32(address, and_value as i64);

        assert_eq!(previous_value, initial_value as i64);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value & and_value);
    }

    #[test]
    fn test_atomic_rmw_sub_i64_to_i32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 16;
        let initial_value: i32 = 100_000;
        let sub_value: i32 = 30_000;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI32;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_sub_i64_to_i32(address, sub_value as i64);

        assert_eq!(previous_value, initial_value as i64);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value - sub_value);
    }

    #[test]
    fn test_atomic_rmw_or_i64_to_i32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 24;
        let initial_value: i32 = 12_000;
        let or_value: i32 = 8_000;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI32;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_or_i64_to_i32(address, or_value as i64);

        assert_eq!(previous_value, initial_value as i64);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value | or_value);
    }

    #[test]
    fn test_atomic_rmw_xor_i64_to_i32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 32;
        let initial_value: i32 = 12_000;
        let xor_value: i32 = 8_000;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI32;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_xor_i64_to_i32(address, xor_value as i64);

        assert_eq!(previous_value, initial_value as i64);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, initial_value ^ xor_value);
    }

    #[test]
    fn test_atomic_rmw_exchange_i64_to_i32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 40;
        let initial_value: i32 = 25_000;
        let exchange_value: i32 = 10_000;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI32;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let previous_value = memory.atomic_rmw_exchange_i64_to_i32(address, exchange_value as i64);

        assert_eq!(previous_value, initial_value as i64);
        let new_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(new_value, exchange_value);
    }

    #[test]
    fn test_atomic_compare_exchange_i32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let initial_value: i32 = 100;
        let expected: i32 = 100;
        let new_value: i32 = 200;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI32;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let result = memory.atomic_compare_exchange_i32(address, expected, new_value);

        assert_eq!(result, initial_value);
        let final_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(final_value, new_value);
    }

    #[test]
    fn test_atomic_compare_exchange_i32_to_i8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 8;
        let initial_value: i8 = 50;
        let expected: i8 = 50;
        let new_value: i8 = 100;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI8;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let result =
            memory.atomic_compare_exchange_i32_to_i8(address, expected as i32, new_value as i32);

        assert_eq!(result, initial_value as i32);
        let final_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(final_value, new_value);
    }

    #[test]
    fn test_atomic_compare_exchange_i32_to_i16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 16;
        let initial_value: i16 = 1000;
        let expected: i16 = 1000;
        let new_value: i16 = 2000;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI16;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let result =
            memory.atomic_compare_exchange_i32_to_i16(address, expected as i32, new_value as i32);

        assert_eq!(result, initial_value as i32);
        let final_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(final_value, new_value);
    }

    #[test]
    fn test_atomic_compare_exchange_i64() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 0;
        let initial_value: i64 = 100_000;
        let expected: i64 = 100_000;
        let new_value: i64 = 200_000;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI64;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let result = memory.atomic_compare_exchange_i64(address, expected, new_value);

        assert_eq!(result, initial_value);
        let final_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(final_value, new_value);
    }

    #[test]
    fn test_atomic_compare_exchange_i64_to_i8() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 8;
        let initial_value: i8 = 50;
        let expected: i8 = 50;
        let new_value: i8 = 100;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI8;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let result =
            memory.atomic_compare_exchange_i64_to_i8(address, expected as i64, new_value as i64);

        assert_eq!(result, initial_value as i64);
        let final_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(final_value, new_value);
    }

    #[test]
    fn test_atomic_compare_exchange_i64_to_i16() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 16;
        let initial_value: i16 = 1_000;
        let expected: i16 = 1_000;
        let new_value: i16 = 2_000;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI16;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let result =
            memory.atomic_compare_exchange_i64_to_i16(address, expected as i64, new_value as i64);

        assert_eq!(result, initial_value as i64);
        let final_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(final_value, new_value);
    }

    #[test]
    fn test_atomic_compare_exchange_i64_to_i32() {
        let mut memory = LinearMemory::new(1);

        let address: i32 = 24;
        let initial_value: i32 = 10_000;
        let expected: i32 = 10_000;
        let new_value: i32 = 20_000;

        let aligned_ptr = memory.memory[address as usize..].as_mut_ptr() as *mut AtomicI32;
        unsafe { (*aligned_ptr).store(initial_value, Ordering::SeqCst) }

        let result =
            memory.atomic_compare_exchange_i64_to_i32(address, expected as i64, new_value as i64);

        assert_eq!(result, initial_value as i64);
        let final_value = unsafe { (*aligned_ptr).load(Ordering::SeqCst) };
        assert_eq!(final_value, new_value);
    }

    #[test]
    fn test_wait32_with_notify() {
        let mut memory = Arc::new(LinearMemory::new(1));
        let barrier = Arc::new(Barrier::new(2));

        let timeout_seconds = 5_000_000_000;
        let atomic_addr = 117;
        let expected_value = 42;

        if let Some(memory_mut) = Arc::get_mut(&mut memory) {
            memory_mut.write_i32(atomic_addr, expected_value);
        }

        let memory_clone = Arc::clone(&memory);
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier_clone.wait();
            memory_clone.wait_i32(atomic_addr, expected_value, timeout_seconds)
        });

        barrier.wait();
        let mut notified_count = 0;
        while notified_count == 0 {
            notified_count = memory.notify(atomic_addr, 1);
        }

        assert_eq!(notified_count, 1);
        let result = handle.join().unwrap();
        assert_eq!(result, 0);
    }

    #[test]
    fn test_wait64_with_notify() {
        let mut memory = Arc::new(LinearMemory::new(1));
        let barrier = Arc::new(Barrier::new(2));

        let timeout_seconds = 5_000_000_000;
        let atomic_addr = 117;
        let expected_value = 42;

        if let Some(memory_mut) = Arc::get_mut(&mut memory) {
            memory_mut.write_i64(atomic_addr, expected_value);
        }

        let memory_clone = Arc::clone(&memory);
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier_clone.wait();
            memory_clone.wait_i64(atomic_addr, expected_value, timeout_seconds)
        });

        barrier.wait();
        let mut notified_count = 0;
        while notified_count == 0 {
            notified_count = memory.notify(atomic_addr, 1);
        }

        assert_eq!(notified_count, 1);
        let result = handle.join().unwrap();
        assert_eq!(result, 0);
    }
}
