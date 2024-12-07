#![allow(clippy::missing_safety_doc)]
use crate::memory::LinearMemory;

#[no_mangle]
pub extern "C" fn alloc(pages: u32) -> *mut LinearMemory {
    let memory = Box::new(LinearMemory::new(pages));
    Box::into_raw(memory)
}

#[no_mangle]
pub unsafe extern "C" fn dealloc(ptr: *mut LinearMemory) {
    if ptr.is_null() {
        return;
    }
    unsafe {
        drop(Box::from_raw(ptr));
    }
}

#[no_mangle]
pub unsafe extern "C" fn grow(ptr: *mut LinearMemory, pages: u32) -> bool {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &mut *ptr
    };
    memory.grow(pages)
}

#[no_mangle]
pub unsafe extern "C" fn copy(
    src_ptr: *const LinearMemory,
    dest_ptr: *mut LinearMemory,
    src_offset: i32,
    dest_offset: i32,
    byte_count: i32,
) {
    let src_memory = unsafe {
        debug_assert!(!src_ptr.is_null(), "Source LinearMemory pointer is null");
        &*src_ptr
    };

    let dest_memory = unsafe {
        debug_assert!(
            !dest_ptr.is_null(),
            "Destination LinearMemory pointer is null"
        );
        &mut *dest_ptr
    };

    src_memory.copy(src_offset, dest_memory, dest_offset, byte_count);
}

#[no_mangle]
pub unsafe extern "C" fn fill(ptr: *mut LinearMemory, offset: i32, byte_count: i32, value: u8) {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &mut *ptr
    };
    memory.fill(offset, byte_count, value);
}

#[no_mangle]
pub unsafe extern "C" fn read_i32(ptr: *mut LinearMemory, address: i32) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.read_i32(address)
}

#[no_mangle]
pub unsafe extern "C" fn read_i32_from_i8(ptr: *mut LinearMemory, address: i32) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.read_i32_from_i8(address)
}

#[no_mangle]
pub unsafe extern "C" fn read_i32_from_i16(ptr: *mut LinearMemory, address: i32) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.read_i32_from_i16(address)
}

#[no_mangle]
pub unsafe extern "C" fn read_i32_from_u8(ptr: *mut LinearMemory, address: i32) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.read_i32_from_u8(address)
}

#[no_mangle]
pub unsafe extern "C" fn read_i32_from_u16(ptr: *mut LinearMemory, address: i32) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.read_i32_from_u16(address)
}

#[no_mangle]
pub unsafe extern "C" fn write_i32(ptr: *mut LinearMemory, address: i32, value: i32) {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &mut *ptr
    };
    memory.write_i32(address, value);
}

#[no_mangle]
pub unsafe extern "C" fn write_i32_to_i8(ptr: *mut LinearMemory, address: i32, value: i32) {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &mut *ptr
    };
    memory.write_i32_to_i8(address, value);
}

#[no_mangle]
pub unsafe extern "C" fn write_i32_to_i16(ptr: *mut LinearMemory, address: i32, value: i32) {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &mut *ptr
    };
    memory.write_i32_to_i16(address, value);
}

#[no_mangle]
pub unsafe extern "C" fn read_i64(ptr: *mut LinearMemory, address: i32) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.read_i64(address)
}

#[no_mangle]
pub unsafe extern "C" fn read_i64_from_i8(ptr: *mut LinearMemory, address: i32) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.read_i64_from_i8(address)
}

#[no_mangle]
pub unsafe extern "C" fn read_i64_from_i16(ptr: *mut LinearMemory, address: i32) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.read_i64_from_i16(address)
}

#[no_mangle]
pub unsafe extern "C" fn read_i64_from_i32(ptr: *mut LinearMemory, address: i32) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.read_i64_from_i32(address)
}

#[no_mangle]
pub unsafe extern "C" fn read_i64_from_u8(ptr: *mut LinearMemory, address: i32) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.read_i64_from_u8(address)
}

#[no_mangle]
pub unsafe extern "C" fn read_i64_from_u16(ptr: *mut LinearMemory, address: i32) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.read_i64_from_u16(address)
}

#[no_mangle]
pub unsafe extern "C" fn read_i64_from_u32(ptr: *mut LinearMemory, address: i32) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.read_i64_from_u32(address)
}

#[no_mangle]
pub unsafe extern "C" fn write_i64(ptr: *mut LinearMemory, address: i32, value: i64) {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &mut *ptr
    };
    memory.write_i64(address, value);
}

#[no_mangle]
pub unsafe extern "C" fn write_i64_to_i8(ptr: *mut LinearMemory, address: i32, value: i64) {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &mut *ptr
    };
    memory.write_i64_to_i8(address, value);
}

#[no_mangle]
pub unsafe extern "C" fn write_i64_to_i16(ptr: *mut LinearMemory, address: i32, value: i64) {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &mut *ptr
    };
    memory.write_i64_to_i16(address, value);
}

#[no_mangle]
pub unsafe extern "C" fn write_i64_to_i32(ptr: *mut LinearMemory, address: i32, value: i64) {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &mut *ptr
    };
    memory.write_i64_to_i32(address, value);
}

#[no_mangle]
pub unsafe extern "C" fn read_f32(ptr: *mut LinearMemory, address: i32) -> f32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.read_f32(address)
}

#[no_mangle]
pub unsafe extern "C" fn write_f32(ptr: *mut LinearMemory, address: i32, value: f32) {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &mut *ptr
    };
    memory.write_f32(address, value);
}

#[no_mangle]
pub unsafe extern "C" fn read_f64(ptr: *mut LinearMemory, address: i32) -> f64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.read_f64(address)
}

#[no_mangle]
pub unsafe extern "C" fn write_f64(ptr: *mut LinearMemory, address: i32, value: f64) {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &mut *ptr
    };
    memory.write_f64(address, value);
}

#[no_mangle]
pub unsafe extern "C" fn atomic_read_i32(ptr: *mut LinearMemory, address: i32) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_read_i32(address)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_read_i32_from_i8(ptr: *mut LinearMemory, address: i32) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_read_i32_from_i8(address)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_read_i32_from_i16(ptr: *mut LinearMemory, address: i32) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_read_i32_from_i16(address)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_read_i32_from_u8(ptr: *mut LinearMemory, address: i32) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_read_i32_from_u8(address)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_read_i32_from_u16(ptr: *mut LinearMemory, address: i32) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_read_i32_from_u16(address)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_write_i32(ptr: *mut LinearMemory, address: i32, value: i32) {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &mut *ptr
    };
    memory.atomic_write_i32(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_write_i32_to_i8(ptr: *mut LinearMemory, address: i32, value: i32) {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &mut *ptr
    };
    memory.atomic_write_i32_to_i8(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_write_i32_to_i16(ptr: *mut LinearMemory, address: i32, value: i32) {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &mut *ptr
    };
    memory.atomic_write_i32_to_i16(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_read_i64(ptr: *mut LinearMemory, address: i32) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_read_i64(address)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_read_i64_from_i8(ptr: *mut LinearMemory, address: i32) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_read_i64_from_i8(address)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_read_i64_from_i16(ptr: *mut LinearMemory, address: i32) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_read_i64_from_i16(address)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_read_i64_from_i32(ptr: *mut LinearMemory, address: i32) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_read_i64_from_i32(address)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_read_i64_from_u8(ptr: *mut LinearMemory, address: i32) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_read_i64_from_u8(address)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_read_i64_from_u16(ptr: *mut LinearMemory, address: i32) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_read_i64_from_u16(address)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_read_i64_from_u32(ptr: *mut LinearMemory, address: i32) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_read_i64_from_u32(address)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_write_i64(ptr: *mut LinearMemory, address: i32, value: i64) {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &mut *ptr
    };
    memory.atomic_write_i64(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_write_i64_to_i8(ptr: *mut LinearMemory, address: i32, value: i64) {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &mut *ptr
    };
    memory.atomic_write_i64_to_i8(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_write_i64_to_i16(ptr: *mut LinearMemory, address: i32, value: i64) {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &mut *ptr
    };
    memory.atomic_write_i64_to_i16(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_write_i64_to_i32(ptr: *mut LinearMemory, address: i32, value: i64) {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &mut *ptr
    };
    memory.atomic_write_i64_to_i32(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_add_i32(
    ptr: *mut LinearMemory,
    address: i32,
    value: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_add_i32(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_and_i32(
    ptr: *mut LinearMemory,
    address: i32,
    value: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_and_i32(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_sub_i32(
    ptr: *mut LinearMemory,
    address: i32,
    value: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_sub_i32(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_or_i32(
    ptr: *mut LinearMemory,
    address: i32,
    value: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_or_i32(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_xor_i32(
    ptr: *mut LinearMemory,
    address: i32,
    value: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_xor_i32(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_exchange_i32(
    ptr: *mut LinearMemory,
    address: i32,
    value: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_exchange_i32(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_add_i32_to_i8(
    ptr: *mut LinearMemory,
    address: i32,
    value: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_add_i32_to_i8(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_and_i32_to_i8(
    ptr: *mut LinearMemory,
    address: i32,
    value: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_and_i32_to_i8(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_sub_i32_to_i8(
    ptr: *mut LinearMemory,
    address: i32,
    value: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_sub_i32_to_i8(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_or_i32_to_i8(
    ptr: *mut LinearMemory,
    address: i32,
    value: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_or_i32_to_i8(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_xor_i32_to_i8(
    ptr: *mut LinearMemory,
    address: i32,
    value: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_xor_i32_to_i8(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_exchange_i32_to_i8(
    ptr: *mut LinearMemory,
    address: i32,
    value: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_exchange_i32_to_i8(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_add_i32_to_i16(
    ptr: *mut LinearMemory,
    address: i32,
    value: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_add_i32_to_i16(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_and_i32_to_i16(
    ptr: *mut LinearMemory,
    address: i32,
    value: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_and_i32_to_i16(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_sub_i32_to_i16(
    ptr: *mut LinearMemory,
    address: i32,
    value: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_sub_i32_to_i16(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_or_i32_to_i16(
    ptr: *mut LinearMemory,
    address: i32,
    value: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_or_i32_to_i16(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_xor_i32_to_i16(
    ptr: *mut LinearMemory,
    address: i32,
    value: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_xor_i32_to_i16(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_exchange_i32_to_i16(
    ptr: *mut LinearMemory,
    address: i32,
    value: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_exchange_i32_to_i16(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_add_i64(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_add_i64(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_and_i64(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_and_i64(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_sub_i64(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_sub_i64(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_or_i64(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_or_i64(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_xor_i64(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_xor_i64(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_exchange_i64(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_exchange_i64(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_add_i64_to_i8(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_add_i64_to_i8(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_and_i64_to_i8(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_and_i64_to_i8(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_sub_i64_to_i8(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_sub_i64_to_i8(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_or_i64_to_i8(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_or_i64_to_i8(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_xor_i64_to_i8(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_xor_i64_to_i8(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_exchange_i64_to_i8(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_exchange_i64_to_i8(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_add_i64_to_i16(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_add_i64_to_i16(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_and_i64_to_i16(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_and_i64_to_i16(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_sub_i64_to_i16(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_sub_i64_to_i16(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_or_i64_to_i16(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_or_i64_to_i16(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_xor_i64_to_i16(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_xor_i64_to_i16(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_exchange_i64_to_i16(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_exchange_i64_to_i16(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_add_i64_to_i32(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_add_i64_to_i32(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_and_i64_to_i32(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_and_i64_to_i32(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_sub_i64_to_i32(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_sub_i64_to_i32(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_or_i64_to_i32(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_or_i64_to_i32(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_xor_i64_to_i32(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_xor_i64_to_i32(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_rmw_exchange_i64_to_i32(
    ptr: *mut LinearMemory,
    address: i32,
    value: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_rmw_exchange_i64_to_i32(address, value)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_compare_exchange_i32(
    ptr: *mut LinearMemory,
    address: i32,
    current: i32,
    new: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_compare_exchange_i32(address, current, new)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_compare_exchange_i32_to_i8(
    ptr: *mut LinearMemory,
    address: i32,
    current: i32,
    new: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_compare_exchange_i32_to_i8(address, current, new)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_compare_exchange_i32_to_i16(
    ptr: *mut LinearMemory,
    address: i32,
    current: i32,
    new: i32,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_compare_exchange_i32_to_i16(address, current, new)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_compare_exchange_i64(
    ptr: *mut LinearMemory,
    address: i32,
    current: i64,
    new: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_compare_exchange_i64(address, current, new)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_compare_exchange_i64_to_i8(
    ptr: *mut LinearMemory,
    address: i32,
    current: i64,
    new: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_compare_exchange_i64_to_i8(address, current, new)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_compare_exchange_i64_to_i16(
    ptr: *mut LinearMemory,
    address: i32,
    current: i64,
    new: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_compare_exchange_i64_to_i16(address, current, new)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_compare_exchange_i64_to_i32(
    ptr: *mut LinearMemory,
    address: i32,
    current: i64,
    new: i64,
) -> i64 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_compare_exchange_i64_to_i32(address, current, new)
}

#[no_mangle]
pub unsafe extern "C" fn atomic_fence(ptr: *mut LinearMemory) {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.atomic_fence()
}

#[no_mangle]
pub unsafe extern "C" fn notify(ptr: *mut LinearMemory, address: i32, count: i32) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.notify(address, count)
}

#[no_mangle]
pub unsafe extern "C" fn wait_i32(
    ptr: *mut LinearMemory,
    address: i32,
    expected: i32,
    timeout: i64,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.wait_i32(address, expected, timeout)
}

#[no_mangle]
pub unsafe extern "C" fn wait_i64(
    ptr: *mut LinearMemory,
    address: i32,
    expected: i64,
    timeout: i64,
) -> i32 {
    let memory = unsafe {
        debug_assert!(!ptr.is_null(), "LinearMemory pointer is null");
        &*ptr
    };
    memory.wait_i64(address, expected, timeout)
}
