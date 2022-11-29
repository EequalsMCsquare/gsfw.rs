pub trait Dirty {
    fn make_dirty(&self);
    fn clear_dirty(&self);
}
