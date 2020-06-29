pub use super::parser_data::*;

use std::cell::Cell;
use super::radix10::*;
use super::drop_row_request::DropRowRequest;
use super::col_row_position_info::ColRowPositionInfo;

///Allows user to see position info from defined TagHandler function
pub enum UserDataMode<'a> {
    RowItem(&'a StateInfo<'a>),
    XmlTag()
}



pub struct CellInfoLocation<'a> {
    pub row: usize,
    pub column: usize,
    pub column_name: Option<String>,
    pub c: &'a Cell<Option<DropRowRequest>>,
    pub ms: &'a Cell<Radix10Rectange>,    
    pub has_dimensions: bool,
    pub dimensions: Option<Radix10Rectange>,    
    pub is_dropping_row: bool,
    pub header_ref: &'a std::collections::HashMap<String,usize>,
    pub was_dimension_call_successfull: &'a Cell<bool>,
}
impl<'a> CellInfoLocation<'a> {
    pub fn new<D>( parser_data: &ColRowPositionInfo<D>, use_column_name: bool, is_reading_range_defined: bool, drop_cell: &'a Cell<Option<DropRowRequest>>, dim_cell: &'a Cell<Radix10Rectange>, headers: &'a std::collections::HashMap<String,usize>, was_ok_dim_cell: &'a Cell<bool> ) -> Self
    where D: Clone + Default + ToString {
        let col_name = match use_column_name {
            true => {
                parser_data.current_column_header.clone()
            },
            false => {
                None
            }
        };
        Self {
            row: parser_data.row_index, column: parser_data.column_index, column_name: col_name,has_dimensions: is_reading_range_defined, c: drop_cell,dimensions: None
            , ms: dim_cell
            , is_dropping_row: false
            , header_ref: headers
            ,was_dimension_call_successfull: was_ok_dim_cell
        }
              
    }
}

///This struct is passed to the user defined TagHandler
pub struct StateInfo<'a> {
    cell_info: CellInfoLocation<'a>,
}

impl<'a> StateInfo<'a> {
    pub fn new( cell_info: CellInfoLocation<'a>) -> Self {
     
        Self {

            cell_info: cell_info
            
            
        }
              
    }
  
    pub fn drop_row(&self, reason: DropRowRequest) {
        self.cell_info.c.set(Some(reason));
        
    }
    pub fn is_dropping_row(&self) -> bool {         
        self.cell_info.is_dropping_row
    }

    pub fn has_dimenstions(&self) -> bool {
        self.cell_info.has_dimensions
    }
    pub fn set_dimensions(&self, dims: String) -> Result<(), Radix10Error> {

        self.cell_info.ms.set(Radix10Rectange::new(dims.as_str())?);
        self.cell_info.was_dimension_call_successfull.set(true);
        Ok( () )
    }
    pub fn get_column_name(&self) -> Option<String> {
        self.cell_info.column_name.clone()
    }

    pub fn get_dimensions(&self) -> Option<Radix10Rectange> {
        if self.cell_info.has_dimensions {
            Some(self.cell_info.ms.get())
        }
        else {
            None
        }
    }
    pub fn get_column_index_by_name( &self, col_name: &str ) -> Option<usize> {
        match self.cell_info.header_ref.contains_key(col_name) {
            true => {
                Some(self.cell_info.header_ref[col_name])
            },
            false => {
                None
            }
        }
        
    }
  

}

