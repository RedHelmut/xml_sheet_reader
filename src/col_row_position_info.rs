use std::cell::Cell;
use super::radix10;
use super::create_parser_error::CreateParserError;

#[derive(Default)]
pub struct ColRowPositionInfo<C>
where C: Clone + Default + ToString {
    pub column_index: usize,
    pub row_index: usize,
    pub in_range: bool,
    pub is_sub_of_row: bool,
    pub is_in_row_range: bool,    
    pub is_first_row: bool,
    pub is_in_col_range: bool,
    pub current_column_header: Option<String>,
    pub drop_it: Cell<bool>,
    pub last_row_index: usize,
    pub is_first_col: bool,
    pub current_column_value: C,
                                 
}

impl<C> ColRowPositionInfo<C> 
where C: Clone + Default + ToString {
    pub fn update_col_index<'a>(&mut self, e: &quick_xml::events::BytesStart<'_>, col_index_attribute_name: std::option::Option<&'a [u8]> ) -> Result<(),CreateParserError> {
        if let Some(attrib_name) = col_index_attribute_name {
            let c:Vec<_> = e.attributes().filter_map( Result::ok ).filter(|x| x.key == attrib_name).collect();
            match c.len() {
                0 => {
                    if !self.is_first_col {
                        self.column_index = self.column_index + 1;
                    }
                    Ok(())
                },
                1 => {
                    match String::from_utf8(c[0].value.clone().into_owned()) {
                        Ok( valid_ref ) => {
                            match radix10::Radix10Rectange::get_real_index(valid_ref.as_ref()) {
                                Ok( radix10::NumberType::Real( (column,_)) ) => {
                                    self.column_index = column;
                                    Ok(())
                                },
                                Ok( radix10::NumberType::Infinite( column ) ) => {
                                    Err(CreateParserError::from( format!("Infinite defined in file not defined for value {}.", column ).as_str()))
                                },
                                Err(err) => {
                                    Err(CreateParserError::from( format!("Error parsing xlsx index {}",err).as_str()))
                                }
                            }
                        },
                        Err( _ ) => {
                            Err(CreateParserError::from("Col index is not a valid utf8 number!"))
                        }
                    }
                },
                _ => {
                    let n = String::from_utf8_lossy(attrib_name);
                    Err(CreateParserError::from( format!("Multiple col attributes of name {}",n).as_str()))
                }
            }
            
        }
        else {
            if !self.is_first_col {
                self.column_index = self.column_index + 1;
            }
            Ok(())
        }
    }
    pub fn update_row_index<'a>(&mut self, e: &quick_xml::events::BytesStart<'_>, row_index_attribute_name: Option<&'a [u8]> ) -> Result<(),CreateParserError> {
        if let Some(attrib_name) = row_index_attribute_name {
            let c:Vec<_> = e.attributes().filter_map( Result::ok ).filter(|x| x.key == attrib_name).collect();
            match c.len() {
                0 => {
                    if !self.is_first_row {
                        self.row_index = self.row_index + 1;
                    }
                    Ok(())
                },
                1 => {
                    match String::from_utf8(c[0].value.clone().into_owned()) {
                        Ok( valid_ref ) => {
                            let parsed = valid_ref.parse::<usize>();
                            match parsed {
                                Ok( valid_parse ) => {
                                    if valid_parse > 0 {
                                        self.row_index = valid_parse - 1;
                                        if self.row_index < self.last_row_index {//we have mandatory row order 0-x because we are in an iterator
                                            Err(CreateParserError::from("Because the rows are out of order."))
                                        }
                                        else {
                                            Ok(())
                                        }
                                    }
                                    else {
                                        Err(CreateParserError::from("Row index canno't be less than 1!"))
                                    }
                                },
                                Err(_) => {
                                    Err(CreateParserError::from("Row index is not a valid number!"))
                                }
                            }
                        },
                        Err( _ ) => {
                            Err(CreateParserError::from("Row index is not a valid utf8 number!"))
                        }
                    }
                },
                _ => {
                    let n = String::from_utf8_lossy(attrib_name);
                    Err(CreateParserError::from( format!("Multiple row attributes of name {}",n).as_str()))
                }
            }
            
        }
        else {
            if !self.is_first_row {
                self.row_index = self.row_index + 1;
            }
            Ok(())
        }
    }

}