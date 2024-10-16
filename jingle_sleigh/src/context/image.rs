use crate::VarNode;
use std::cmp::min;
use std::iter::once;
use std::ops::Range;

pub trait ImageProvider {
    fn load(&self, vn: &VarNode, output: &mut [u8]) -> usize;

    fn has_full_range(&self, vn: &VarNode) -> bool;
}

pub trait ImageProviderExt: ImageProvider {
    fn get_section_info(&self) -> impl Iterator<Item=ImageSection>;
}

impl ImageProvider for &[u8] {
    fn load(&self, vn: &VarNode, output: &mut [u8]) -> usize {
        //todo: check the space. Ignoring for now
        let vn_range: Range<usize> = Range::from(vn);
        let vn_range = Range {
            start: vn_range.start,
            end: min(vn_range.end, self.len()),
        };
        if let Some(s) = self.get(vn_range) {
            if let Some(o) = output.get_mut(0..s.len()) {
                o.copy_from_slice(s)
            }
            let o_len = output.len();
            if let Some(o) = output.get_mut(s.len()..o_len) {
                o.fill(0);
            }
            s.len()
        } else {
            output.fill(0);
            0
        }
    }

    fn has_full_range(&self, vn: &VarNode) -> bool {
        let vn_range: Range<usize> = Range::from(vn);
        vn_range.start < self.len() && vn_range.end <= self.len()
    }
}

impl ImageProviderExt for &[u8] {
    fn get_section_info(&self) -> impl Iterator<Item=ImageSection> {
        once(ImageSection {
            data: &self,
            base_address: 0,
            perms: Perms {
                read: true,
                write: false,
                exec: true,
            },
        })
    }
}

impl ImageProvider for Vec<u8> {
    fn load(&self, vn: &VarNode, output: &mut [u8]) -> usize {
        self.as_slice().load(vn, output)
    }

    fn has_full_range(&self, vn: &VarNode) -> bool {
        self.as_slice().has_full_range(vn)
    }
}

impl ImageProviderExt for Vec<u8> {
    fn get_section_info(&self) -> impl Iterator<Item=ImageSection> {
        once(ImageSection {
            data: &self,
            base_address: 0,
            perms: Perms {
                read: true,
                write: false,
                exec: true,
            },
        })
    }
}

#[derive(Debug, Clone)]
pub struct Perms {
    pub(crate) read: bool,
    pub(crate) write: bool,
    pub(crate) exec: bool,
}

impl Perms {
    pub const RWX: Perms = Perms {
        read: true,
        write: true,
        exec: true,
    };
    pub const RX: Perms = Perms {
        read: true,
        write: false,
        exec: true,
    };

    pub const RW: Perms = Perms {
        read: true,
        write: true,
        exec: false,
    };
    pub const R: Perms = Perms {
        read: true,
        write: false,
        exec: false,
    };

    pub const NONE: Perms = Perms {
        read: false,
        write: false,
        exec: false,
    };
}

#[derive(Debug, Clone)]
pub struct ImageSection<'a> {
    pub(crate) data: &'a [u8],
    pub(crate) base_address: usize,
    pub(crate) perms: Perms,
}
