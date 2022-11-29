use std::cell::Cell;

#[derive(Clone)]
pub struct DirtyMark(Cell<bool>);

unsafe impl Send for DirtyMark {}
unsafe impl Sync for DirtyMark {}

impl DirtyMark {
    pub fn make_dirty(&self) {
        self.0.set(true)
    }
    pub fn clear_dirty(&self) {
        self.0.set(false)
    }
}

impl std::fmt::Display for DirtyMark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.get())
    }
}

impl std::fmt::Debug for DirtyMark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("DirtyMark").field(&self.0).finish()
    }
}

impl Default for DirtyMark {
    fn default() -> Self {
        Self(false.into())
    }
}
