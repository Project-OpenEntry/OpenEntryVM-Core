use std::{fs::File, collections::HashMap, io::Read, sync::Arc};

use tar::{Archive as Tar, Entry};
use flate2::read::GzDecoder;

use crate::{vm_config::VMConfig, block_info::BlockInfo};

fn read_entry<'a>(readable: &mut Entry<'a, GzDecoder<File>>) -> Box<[u8]> {
    let mut vec = Vec::with_capacity(readable.header().size().unwrap() as usize);

    readable.read_to_end(&mut vec).unwrap();

    vec.into_boxed_slice()
}

pub struct Archive {
    pub block_info: Option<Arc<BlockInfo>>,
    pub files: HashMap<String, Box<[u8]>>,
    pub code: Box<[u8]>,
    pub conf: VMConfig,
}

impl Archive {
    pub fn open(path: impl Into<String>) -> Archive {
        let file = File::open(path.into());

        if let Ok(file) = file {
            let tar = GzDecoder::new(file);
            let mut tar = Tar::new(tar);
            let mut entries = tar
                .entries()
                .unwrap()
                .map(|x| x.unwrap())
                .fold(HashMap::new(), |mut acc, mut x| {
                    acc.insert(x.path().unwrap().to_mut().clone().into_os_string().into_string().unwrap(), read_entry(&mut x));

                    acc
                });

            Archive {
                code: entries.remove("Main.bin").unwrap(),
                conf: VMConfig::read(entries.remove("Conf.bin").unwrap()),
                block_info: entries.remove("BlockInfo.bin").map(|buffer| BlockInfo::read(buffer)),
                files: entries
            }
        } else {
            panic!("Cannot open file.")
        }
    }
}