use super::radix10::{Radix10Rectange, Radix10Error};
use std::cell::Cell;

pub struct NonRowInfo<'a> {
    has_dimensions: bool,
    dimensions: &'a Cell<Radix10Rectange>,
    changed_dims: &'a Cell<bool>,
}
impl<'a> NonRowInfo<'a> {
    pub fn new(dim_cell: &'a Cell<Radix10Rectange>, has_dims: bool, changed_dims_ok: &'a Cell<bool>) -> Self {
        Self {
            dimensions: dim_cell,
            has_dimensions: has_dims,
            changed_dims: changed_dims_ok,
        }
    }
    pub fn get_dimensions(&self) -> Option<Radix10Rectange> {
        if self.has_dimensions {
            Some(self.dimensions.get())
        }
        else {
            None
        }
    }
    pub fn set_dimensions(&self, dims: String) -> Result<(), Radix10Error> {
        self.dimensions.set(Radix10Rectange::new(dims.as_str())?);
        self.changed_dims.set( true);
        Ok( () )
    }
}