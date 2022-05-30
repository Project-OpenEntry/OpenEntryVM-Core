macro_rules! gen_enum {
    ($name: ident, $type: ty, [$($key: ident = $value: literal,)*]) => {
        #[allow(non_snake_case, non_upper_case_globals)]
        pub mod $name {
            $(pub const $key: $type = $value;)*
        }
    }
}

pub trait ReadBuffer {
    fn const_read<const OFFSET: isize, T: Copy>(&self) -> T;
    fn read<T: Copy>(&self, offset: isize) -> T;
}

impl ReadBuffer for Box<[u8]> {
    fn const_read<const OFFSET: isize, T: Copy>(&self) -> T {
        unsafe { std::ptr::read::<T>(self.as_ptr().offset(OFFSET).cast::<T>()) }
    }

    fn read<T: Copy>(&self, offset: isize) -> T {
        unsafe { std::ptr::read::<T>(self.as_ptr().offset(offset).cast::<T>()) }
    }
}

pub(crate) use gen_enum;

use crate::executor::executor::Lock;

#[inline]
pub fn handle_lock<const DROP: bool>(lock: Lock) -> Lock {
    if DROP {
        drop(lock);

        None
    } else {
        lock
    }
}