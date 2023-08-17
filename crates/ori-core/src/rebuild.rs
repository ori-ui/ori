pub use ori_macro::Rebuild;

use crate::RebuildCx;

pub trait Rebuild {
    fn rebuild(&self, cx: &mut RebuildCx, old: &Self);
}
