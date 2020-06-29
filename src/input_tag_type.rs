use std::collections::HashMap;
use super::row_validation::RowValidation;
use super::state_info::StateInfo;
use super::non_row_info::NonRowInfo;
use super::header_complete_handler::HeaderCompleteHandler;

pub enum InputTagType<'b,C> 
where C: Clone + Default + ToString {
    ColumnStart(&'b mut C, HashMap<String, String>, &'b StateInfo<'b>),
    ColumnEnd(&'b mut C, &'b StateInfo<'b>),
    ColumnText(&'b mut C, &'b str, &'b StateInfo<'b>),
    ColumnHeader(&'b mut C, &'b mut String, &'b mut HashMap<String,String>, &'b StateInfo<'b>),
    HeaderComplete( HashMap<std::string::String, usize>, &'b mut HeaderCompleteHandler ),
    OtherTagStart(HashMap<String, String>, NonRowInfo<'b>),
    OtherTagEnd(NonRowInfo<'b>),
    OtherTagText(&'b str, NonRowInfo<'b>),
    RowComplete( &'b mut RowValidation<C> ),
    RowStart(HashMap<String, String>),
}
