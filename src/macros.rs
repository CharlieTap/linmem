#[macro_export]
macro_rules! make_readers {
    ($($item:tt),* $(,)?) => {
        $(make_readers!(@single $item);)*
    };

    (@single ($read_type:ty)) => {
        paste! {
            make_readers!(@single ([<read_ $read_type>], $read_type, $read_type));
        }
    };

    (@single ($read_type:ty, $address_type:ty)) => {
        paste! {
            make_readers!(@single ([<read_ $read_type _from_ $address_type>], $read_type, $address_type));
        }
    };

    (@single ($fn_name:ident, $read_type:ty, $address_type:ty)) => {
        #[must_use]
        pub fn $fn_name(&self, address: i32) -> $read_type {
            const BYTE_COUNT: usize = size_of::<$address_type>();
            // Safety we assume the params passed are correct
            unsafe {
                let pointer = self.memory.as_ptr().add(address as usize).cast::<[u8; BYTE_COUNT]>();
                <$address_type>::from_le_bytes(std::ptr::read_unaligned(pointer)) as $read_type
            }
        }
    };

    (@single (@atomic $read_type:ty, $address_type:ty)) => {
        paste! {
            make_readers!(@single (@atomic [<atomic_read_ $read_type>],
                             $read_type, $address_type, $read_type));
        }
    };

    (@single (@atomic $read_type:ty, $address_type:ty, $address_type_non_atomic: ty)) => {
        paste! {
            make_readers!(@single (@atomic [<atomic_read_ $read_type _from_ $address_type_non_atomic>],
                             $read_type, $address_type, $address_type_non_atomic));
        }
    };

    (@single (@atomic $fn_name:ident, $read_type:ty, $address_type:ty, $address_type_non_atomic: ty)) => {
        #[must_use]
        pub fn $fn_name(&self, address: i32) -> $read_type {
            // Safety we assume the params passed are correct
            unsafe {
                let pointer = self.memory.as_ptr().add(address as usize).cast::<$address_type>();
                (*pointer).load(Ordering::SeqCst) as $read_type
            }
        }
    };
}

#[macro_export]
macro_rules! make_writers {
    ($($item:tt),* $(,)?) => {
        $(make_writers!(@single $item);)*
    };

    (@single ($write_type:ty)) => {
        paste! {
            make_writers!(@single ([<write_ $write_type>], $write_type, $write_type));
        }
    };

    (@single ($write_type:ty, $address_type:ty)) => {
        paste! {
            make_writers!(@single ([<write_ $write_type _to_ $address_type>], $write_type, $address_type));
        }
    };

    (@single ($fn_name:ident, $write_type:ty, $address_type:ty)) => {
        pub fn $fn_name(&mut self, address: i32, value: $write_type) {
            const BYTE_COUNT: usize = size_of::<$address_type>();
            // Safety we assume the params passed are correct
            unsafe {
                let write_val = (value as $address_type).to_le_bytes();
                let pointer = self.memory.as_mut_ptr().add(address as usize).cast::<[u8; BYTE_COUNT]>();
                std::ptr::write_unaligned(pointer, write_val)
            }
        }
    };

    (@single (@atomic $write_type:ty, $address_type:ty)) => {
        paste! {
            make_writers!(@single (@atomic [<atomic_write_ $write_type>],
                             $write_type, $address_type, $write_type));
        }
    };

    (@single (@atomic $write_type:ty, $address_type:ty, $address_type_non_atomic: ty)) => {
        paste! {
            make_writers!(@single (@atomic [<atomic_write_ $write_type _to_ $address_type_non_atomic>],
                             $write_type, $address_type, $address_type_non_atomic));
        }
    };

    (@single (@atomic $fn_name:ident, $write_type:ty, $address_type:ty, $address_type_non_atomic: ty)) => {
        pub fn $fn_name(&self, address: i32, value: $write_type) {
            // Safety we assume the params passed are correct
            unsafe {
                let pointer = self.memory.as_ptr().add(address as usize).cast::<$address_type>();
                (*pointer).store(value as $address_type_non_atomic, Ordering::SeqCst);
            }
        }
    };
}
