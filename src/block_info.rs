use std::{collections::HashMap, sync::Arc};

use crate::utils::ReadBuffer;

pub enum UnlockInfo {
    Current,
    Addr(u64)
}

pub struct BlockInfo(HashMap<u64, UnlockInfo>);

impl BlockInfo {
    pub fn read(buffer: Box<[u8]>) -> Arc<BlockInfo> {
        let size = buffer.const_read::<0, u64>();
        let mut map = HashMap::new();

        for i in (0..size).map(|i| i as isize * 16 + 8) {
            let lock_start = buffer.read::<u64>(i);
            let lock_end = buffer.read::<u64>(i + 8);

            if lock_start == lock_end {
                map.insert(lock_start, UnlockInfo::Current);
            } else {
                map.insert(lock_start, UnlockInfo::Addr(lock_end));
            }
        }

        Arc::new(BlockInfo(map))
    }

    pub fn get(&self, inst: u64) -> Option<&UnlockInfo> {
        self.0.get(&inst)
    }
}