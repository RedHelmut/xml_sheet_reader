use quick_xml::Reader;  
use quick_xml::events::Event;
use std::cell::Cell;
use super::radix10;
use super::create_parser_error::*;
pub use super::state_info::*;
use super::tag_handler::*;
use super::helpers;

use super::parser_data::{ParserData};
use super::drop_row_request::DropRowRequest;
use super::row_validation::RowValidation;
pub use super::parser_flags::ParserFlags;
use super::xml_reading_range::XmlReadingRange;
use super::completion::Completion;

macro_rules! serve_empty_lines_if_any {
    ($s:expr, $ret: expr) => {
        if $s.parser_data.holding_for_x_blank_lines > 0 {
            $ret = Some(RowValidation::Valid(vec![C::default();$s.parser_data.reading_range.column_count]));
            $s.parser_data.holding_for_x_blank_lines = $s.parser_data.holding_for_x_blank_lines - 1;
            break;
        }
    };
}


///* Main function used to parse an xml document using both internal and user defined functions.
///* User functions must impliment trait ModColumnData<DesiredReturnType>
///* Factory function is CreateParser::new(...)
pub struct CreateParser<'a,'b,S,C>
where S: Sized,
C: Clone + Default + ToString {
    parser_data: ParserData<'b,S,C>,
    f_reader: quick_xml::Reader<&'a [u8]>,
    has_critical_error: bool,
    
}
impl<'a,'b,S,C> CreateParser<'a,'b,S,C>
where S: Sized,
C: Clone + Default + ToString {
    pub fn add( &mut self, f: TagHandler<'b,S,C> ) {
        let name = f.n.clone();
        self.parser_data.k.entry(name.name).or_insert(f);
    }
    pub fn add_many( &mut self, j: Vec<TagHandler<'b,S,C>> ) {
        for f in j {
            self.add(f);
        }
    }
    
   /** # Arguments
 * 'sheet_file_buffer' - buffer containing the sheet in memory
 * 'range':XmlReadingRange - enum which allows for known or unknwon spreadsheet range such as XmlReadingRange::Defined('A1:B44') or XmlReadingRange:Unknown which must be found before the first 'row' tag
 * 's': a mut ref to a instance of a struct that impliments ModColumnData 
 * 'row_tag': &[u8] - is the row that the iterator will break on end tag 'worksheet/sheetrows/row|r|' if there is an attribute tag it will use it to get the row number from ommited with increment every row
 * 'column_tag' : Option<&[u8]> - optional column tag if there is none then row will only return one value
 * 'parser_flags' : ParserFlags - used to control how rows are read and header information is handled
# Example
     #[derive(Clone)]
     pub struct Share<C>
     where C: Clone + Default + ToString {
         col_data: C,
         last_col_type_attrib: String,
         shared_strings: Vec<String>,
         dimensions: Option<String>,
         table_data: Table,
     }
     impl<C> Share<C>
     where C: Clone + Default + ToString {
         pub fn new() -> Self {
            Self {
                 col_data: C::default(),
                 last_col_type_attrib: String::default(),
                 shared_strings: Vec::new(),
                 dimensions: None,
                 table_data: Table::default(),
             }
         }    
     }
     impl<C> ModColumnData<C> for Share<C>
     where C: Clone + Default + ToString {
         fn get_value(&self) -> C {
             self.col_data.clone()
         }
         fn set_value(&mut self, col_data: C) {
             self.col_data = col_data
         }
         fn get_dimensions(&self) -> Option<String> {
             self.dimensions.clone()
         }
     }    
     #[derive(Default, Clone)]
     pub struct ColumnData {
     pub value: String,
     pub saved_attribs: HashMap<String,String>,
     }
     impl ColumnData {
         pub fn new( val: String, val_type: HashMap<String,String> ) -> Self {
             Self {
                 value: val,
                 saved_attribs: val_type
             }
         }        
     }
     fn main() {
         let reading_range = XmlReadingRange::Defined(mmm.table_info.table_dimensions.as_str());
         let mut share: Share<Column> = Share::new()
         let row_tag = b"worksheet/sheetData/row|r|";
         let col_tag = Some(b"worksheet/sheetData/row/c|r|");
         let flags = ParserFlags::ReturnHeader 
                     | ParserFlags::HasHeader 
                     | ParserFlags::ModifyHeader 
                     | ParserFlags::RowsInSequentialOrderCanSkip;
         xlsx::create_parser::CreateParser::new(astd.as_str(), reading_range, &mut sh, row_tag, col_tag, flags );
    
            let c = TagHandler::new(b"worksheet/sheetData/row/c/v", |x: &mut Share<ColumnData>, tag_type, cell_info| {
                match tag_type {
                InputTagType::Start(attribs) => {
                },
                InputTagType::End => {
                },                
                InputTagType::Text(s) => {
                    //x.col_data = ColumnData::new(s.to_owned(), x..clone())
                    x.col_data.value = s.to_owned();
                    match cell_info.cell_info.column_name {
                        Some(name) => {
                            match name.as_ref() {
                                "Qty" => {
                                    match x.col_data.value.parse::<i64>() {
                                        Ok( parsed_data ) => {
                                            x.total_qty = x.total_qty + parsed_data;
                                        },
                                        Err(e) => {                                        
                                        }
                                    }                                    
                                },                                       
                                _ => {
                                }
                            }
                        },
                        None => {
                        }
                    }                    
                },
                _ => {
                }
            }
        }); 
    }
    
*/
    pub fn new(sheet_file_buffer: &'a str, range: XmlReadingRange, s: &'b mut S, row_tag: &'b [u8], column_tag: Option<&'b [u8]>, parser_flags: ParserFlags) -> Result<Self, Box<dyn std::error::Error>> {
        
        let optional_row_id = helpers::get_relative_name_parts(row_tag);
        let col_header_tag = 
            match column_tag {
                Some( col_tag ) => {
                    let ct = helpers::get_relative_name_parts(col_tag);
                    if !(ct.name.starts_with(optional_row_id.name) && optional_row_id.name.len() < ct.name.len()) {
                        return Err(  Box::new(CreateParserError::from("Make new error type but the column tag needs to be a subset of the row tag ") ));
                    }        

                    Some(ct)
                },
                None => {
                    None
                }
            };
       
        let mut temp_reader = Reader::from_str(sheet_file_buffer);
        let f_reader = temp_reader.expand_empty_elements(true);
        f_reader.trim_text(true);
        

        let read_ran = 
            match range {
                XmlReadingRange::Defined( range_val ) => {
                    Some(radix10::Radix10Rectange::new(range_val)?)
                },
                XmlReadingRange::WillDefineBeforeRows => {
                    None
                }
            };        

        Ok(CreateParser {
            f_reader: temp_reader,
            parser_data: ParserData::new(s, read_ran, optional_row_id, col_header_tag, parser_flags ),
            has_critical_error: false,
        })

        
    }


}



impl<'a,'b,S,C> Iterator for CreateParser<'a,'b,S,C> 
where C: Clone + Default + ToString {
    type Item = RowValidation<C>;

    
    ///Gets next row tag with optional column if it contains more than one value.
    fn next( &mut self ) -> Option<Self::Item> {  

        let mut buf: Vec<_> = Vec::new();        
        let mut ret: Option<Self::Item> = None;
        let mut row_data: Vec<(usize,C)> = Vec::new();
        let f_r = self.f_reader.expand_empty_elements(true);
        f_r.trim_text(true);
        let mut drop_it: Cell<Option<DropRowRequest>> = Cell::new(None);
        let mut just_getting_the_header = false;
        if self.has_critical_error {
            return None;                     
        }


        loop {
           
        
            //serve the blank lines that dont exist in xml document, macro will break and serve line if any.
            serve_empty_lines_if_any!(self, ret);

            let event_read = f_r.read_event(&mut buf);
            
            match event_read {                             
                Ok(Event::Start(ref e) ) => {  

                    match self.parser_data.run_start_tags(e, &mut just_getting_the_header,&mut drop_it) {
                        Completion::EndOfRows => {
                            ret = None;
                            break;
                        },
                        Completion::CriticalFail( critical_error ) => {
                            self.has_critical_error = true;
                            ret = Some(RowValidation::CriticalFail( critical_error));
                            break;
                        },
                        Completion::RowFail( fail_row ) => {
                            ret = Some( fail_row );        
                            break;
                        },
                        _ => {

                        } 
                    }
                },
                Ok(Event::Text( ref e) ) => {      
                     match self.parser_data.run_text_tags(e, &mut just_getting_the_header,&mut drop_it, f_r) {
                        Completion::OkWithRow(rw) => {
                            ret = Some(rw);
                            break;
                        },
                        Completion::CriticalFail(f) => {
                            self.has_critical_error = true;
                            ret = Some(RowValidation::CriticalFail( f));
                            break;
                        },
                        _ => {
                            
                        }
                    }
                },
                Ok(Event::End(e) ) => {
                    match self.parser_data.run_end_tags(e, &mut just_getting_the_header,&mut drop_it, &mut row_data) {
                        Completion::OkWithRow(rw) => {
                            ret = Some(rw);
                            break;
                        },
                        Completion::CriticalFail(f) => {
                            self.has_critical_error = true;
                            ret = Some(RowValidation::CriticalFail( f));
                            break;
                        },
                        Completion::EndOfRows => {
                            ret = None;
                            break;
                        },                        
                        _ => {
                            
                        }
                    }
 
                },
                Ok(Event::Eof) => {       
                    ret = None;            
                    break
                },
                Err(e) => {
                    let err = format!("{}",e);
                    ret = Some(RowValidation::Invalid(CreateParserError::from(err.as_str())));            
                    break
                },
                _ => {
                    ()
                }
            }
        }
        ret
    }

}