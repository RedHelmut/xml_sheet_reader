use super::helpers;
use super::input_tag_type::{InputTagType};

pub struct TagHandler<'a,SharedInfo,ColumnReturn> 
where SharedInfo: Sized,
ColumnReturn: Clone + Default + ToString {
    pub f: Box<dyn Fn(&mut SharedInfo,InputTagType<ColumnReturn>)>,
    pub n: helpers::RelativeNameParts<'a>,
}

impl<'a,SharedInfo,ColumnReturn> TagHandler<'a,SharedInfo,ColumnReturn>
where SharedInfo: Sized,
ColumnReturn: Clone + Default + ToString {

    pub fn new<F>( s: &'a [u8], f: F ) -> Self
    where F: Fn(&mut SharedInfo,InputTagType<ColumnReturn>) + 'static {    
        let rel = helpers::get_relative_name_parts( s );
        TagHandler {
            f: Box::new(f),
            n: rel,
        }
    }
}