use super::radix10;
use std::cell::Cell;
use std::collections::HashMap;

use super::tag_handler::*;
use super::helpers;
use super::parser_flags::*;
use super::create_parser_error::CreateParserError;
use super::state_info::*;
use super::row_validation::RowValidation;
use super::input_tag_type::{InputTagType};
use super::col_row_position_info::ColRowPositionInfo;
use super::drop_row_request::DropRowRequest;
use super::completion::Completion;
use super::current_tag_line::CurrentTagLine;
use super::non_row_info::NonRowInfo;
use super::header_complete_handler::HeaderCompleteHandler;


///Stores data used to transverse the CreateParser struct
pub struct ParserData<'a,S,C>
where C: Clone + Default + ToString {
    pub k: HashMap<&'a [u8],TagHandler<'a,S,C>>,     
    pub cell: &'a mut S,//&'a mut
    pub reading_range: radix10::Radix10Rectange,
    pub current_tag_line: CurrentTagLine,
    pub column_tag: Option<&'a [u8]>,
    pub row_tag: &'a [u8],
    pub row_tag_string: String,
    pub row_index_attribute_name: Option<&'a [u8]>,
    pub col_index_attribute_name: Option<&'a [u8]>,
    pub holding_for_x_blank_lines: usize,
    pub header_row: HashMap<String, usize>,
    pub header_row_vec: Vec<String>,
    pub drop_row_request: Option<DropRowRequest>,    
    pub col_row_pos_info: ColRowPositionInfo<C>,
    pub is_reading_range_defined: bool,
    pub header_attributes: HashMap<String,String>,
    pub parse_flags: ParserFlags,
    pub can_change_range: bool,
    pub call_end_row_user_func: bool,
    pub last_header_string: String,

}


impl<'a,S,C> ParserData<'a,S,C> 
where C: Clone + Default + ToString {
    pub fn new( s: &'a mut S, read_range: Option<radix10::Radix10Rectange>, optional_row_id: helpers::RelativeNameParts<'a>, optional_col_id: Option<helpers::RelativeNameParts<'a>>, parser_flags: ParserFlags ) -> Self {
        let row_index_id = 
            if optional_row_id.attribs.len() == 1 {
                Some(optional_row_id.attribs[0].attrib)
            }
            else {
                None
            };
       let col_index_id = 
            match optional_col_id.clone() {
                Some( col_rel ) => {
                    if col_rel.attribs.len() == 1 {
                        Some(col_rel.attribs[0].attrib)
                    }
                    else {
                        None
                    }
                },
                None => {
                    None
                }
            };
        let col_name_to_pass = 
            match optional_col_id {
                Some( col_name_opt ) => {
                    let vl = col_name_opt.clone().name;
                    Some( vl )
                },
                None => {
                    None
                }
                
            };


        let (range_def, is_range_def) = 
            if read_range.is_some() {
                (read_range.unwrap(), true)
            }
            else {
                (radix10::Radix10Rectange::zero(),false)
            };

        Self {
            k: HashMap::new(),
            cell: s,
            reading_range: range_def,
            is_reading_range_defined: is_range_def,        
            current_tag_line: CurrentTagLine{ tag_line: String::new(),last_tag_sizes: Vec::new() },
            column_tag: col_name_to_pass,
            row_tag: optional_row_id.name,
            row_tag_string: String::from_utf8_lossy(optional_row_id.name).into_owned(),
            row_index_attribute_name: row_index_id,
            col_index_attribute_name: col_index_id,
            holding_for_x_blank_lines: 0,
            header_row: HashMap::new(),       
            drop_row_request: None,
            col_row_pos_info: ColRowPositionInfo::default(),
            header_attributes: HashMap::new(),
            parse_flags: parser_flags,
            can_change_range: true,
            header_row_vec: Vec::new(),      
            call_end_row_user_func: false,  
            last_header_string: String::new(),
        }
    }

    ///Gets largest amount if rows skip with ParserFlag::RowsInSequentialOrderCanSkip set.CreateParserError
    ///If row id is 14 and last row was 10 but reading range is 12 then it will return 12 as max.
    pub fn get_largest_avalable_empty_lines( parser_data: &ParserData<S,C>) -> usize {
        let largest =
            if !parser_data.reading_range.is_row_count_infinite { 
                if parser_data.col_row_pos_info.row_index > parser_data.reading_range.row + parser_data.reading_range.row_count {
                    parser_data.reading_range.row + parser_data.reading_range.row_count
                }
                else {
                    parser_data.col_row_pos_info.row_index
                }
            }
            else {
                parser_data.col_row_pos_info.row_index
            };
        largest
    }
 


    ///Helper function that just updates the boolean range values.
    pub fn update_range_items(&mut self) {
        self.col_row_pos_info.is_in_row_range = self.col_row_pos_info.row_index >= self.reading_range.row && (( self.col_row_pos_info.row_index < self.reading_range.row + self.reading_range.row_count ) || self.reading_range.is_row_count_infinite);
        self.col_row_pos_info.is_in_col_range = self.col_row_pos_info.column_index >= self.reading_range.column && self.col_row_pos_info.column_index < self.reading_range.column + self.reading_range.column_count;
        self.col_row_pos_info.in_range = self.col_row_pos_info.is_in_row_range && self.col_row_pos_info.is_in_col_range;
        self.col_row_pos_info.is_sub_of_row = self.current_tag_line.tag_line.starts_with(self.row_tag_string.as_str());

    }

    pub fn update_for_missing_rows(&mut self) {
        
        if self.col_row_pos_info.row_index > self.col_row_pos_info.last_row_index + 1{
            //there is a spacing in the xml for sequential rows so add some blank lines to the que at top.
            
            let in_range = self.col_row_pos_info.row_index >= self.reading_range.row 
                        && (( self.col_row_pos_info.row_index < self.reading_range.row + self.reading_range.row_count ) 
                        || self.reading_range.is_row_count_infinite);
            //make sure that we don't go over the row request size.
            let largest = ParserData::get_largest_avalable_empty_lines(&self);
            
            if in_range {
                self.holding_for_x_blank_lines = largest - self.col_row_pos_info.last_row_index;
            }
        }
    }

    pub fn is_row_greater_than_reading_range(&self) -> bool {
        ( self.col_row_pos_info.row_index > self.reading_range.row + self.reading_range.row_count ) && !self.reading_range.is_row_count_infinite 
    }

    pub fn get_current_column_name_by_column_index( &self ) -> Option<String> {
        if self.col_row_pos_info.is_in_col_range {
            let vvv = self.col_row_pos_info.column_index - self.reading_range.column;
            if vvv < self.header_row.len() {
                
                Some(self.header_row_vec[vvv].clone())
            }
            else {
                None
            }
        }
        else {
            None
        }        
    }

    pub fn run_start_user_tag(&mut self, row_tag_id: &[u8], e: &quick_xml::events::BytesStart<'_>, drop_it: &mut Cell<Option<DropRowRequest>>, dims: &mut Cell<radix10::Radix10Rectange> ) {
        match self.k.contains_key(row_tag_id) {
            true => {
                let hsh = Self::hash_attribs_that_exist_both_user_and_xml( &self.k[row_tag_id].n.attribs, e );
                let was_ok_reading_range_rewrite: Cell<bool> = Cell::new(false);
                let mut sti = StateInfo::new(CellInfoLocation::new(&self.col_row_pos_info, true, self.is_reading_range_defined, drop_it, &dims, &self.header_row, &was_ok_reading_range_rewrite ));                                                
              //  let mut passed_user_data_mode = UserDataMode::RowItem(&mut sti);
                
                if self.row_tag == row_tag_id {
                    (self.k[row_tag_id].f)( &mut self.cell, InputTagType::RowStart(hsh));                                        
                }
                else {
                    (self.k[row_tag_id].f)( &mut self.cell, InputTagType::ColumnStart(&mut self.col_row_pos_info.current_column_value,hsh, &mut sti));
                }

                
                if self.can_change_range {
                    if was_ok_reading_range_rewrite.get() == true {
                
                        self.reading_range = dims.get();
                        self.is_reading_range_defined = true;
                    }
                }
                self.drop_row_request = (*drop_it.get_mut()).clone();
            },
            false => {
            }
        }                            
    
    }

  
    pub fn run_start_user_tag_non_row(&mut self, row_tag_id: &[u8], e: &quick_xml::events::BytesStart<'_>, dims: &mut Cell<radix10::Radix10Rectange>  ) {
        match self.k.contains_key(row_tag_id) {
            true => {
                let hsh = Self::hash_attribs_that_exist_both_user_and_xml( &self.k[row_tag_id].n.attribs, e );
                //let mut dim_try = Dimensions{dims: None, has_dimensions_query: self.is_reading_range_defined  };
                let was_ok_reading_range_rewrite: Cell<bool> = Cell::new(false);
           
                let info = NonRowInfo::new(&dims, self.is_reading_range_defined, &was_ok_reading_range_rewrite );
                (self.k[row_tag_id].f)( &mut self.cell, InputTagType::OtherTagStart(hsh, info));                                        
                if self.can_change_range {
                    if was_ok_reading_range_rewrite.get() == true {
                
                        self.reading_range = dims.get();
                        self.is_reading_range_defined = true;                    
                    }
                }
                
            },
            false => {
            }
        }                            
    
    }
    pub fn run_end_user_tag_non_row(&mut self, row_tag_id: &[u8], _: quick_xml::events::BytesEnd<'_>, dims: &mut Cell<radix10::Radix10Rectange>  ) {
        match self.k.contains_key(row_tag_id) {
            true => {
                let was_ok_reading_range_rewrite: Cell<bool> = Cell::new(false);
           
                let info = NonRowInfo::new(&dims, self.is_reading_range_defined, &was_ok_reading_range_rewrite );
                (self.k[row_tag_id].f)( &mut self.cell, InputTagType::OtherTagEnd(info));                                        
                if self.can_change_range {
                    if was_ok_reading_range_rewrite.get() == true {
                        self.reading_range = dims.get();
                        self.is_reading_range_defined = true;          
                    }          
                }
                
            },
            false => {
            }
        }                            
    
    }

    pub fn run_text_user_tag<'c>(&mut self, row_tag_id: &[u8], e: &quick_xml::events::BytesText<'_>, drop_it: &mut Cell<Option<DropRowRequest>>,f_r: &mut quick_xml::Reader<&'c [u8]>) {
        match self.k.contains_key(row_tag_id) {
            true => {
                let text = e.unescape_and_decode(&f_r).unwrap();

                let dims: Cell<radix10::Radix10Rectange> = Cell::new(self.reading_range);
                let was_ok_reading_range_rewrite: Cell<bool> = Cell::new(false);
                let mut sti = StateInfo::new(CellInfoLocation::new(&self.col_row_pos_info, true, self.is_reading_range_defined, drop_it, &dims, &self.header_row, &was_ok_reading_range_rewrite ));                   

                if self.row_tag == row_tag_id {                                            
                    (self.k[row_tag_id].f)( &mut self.cell, InputTagType::ColumnText( &mut self.col_row_pos_info.current_column_value,text.as_ref(), &mut sti ) );
                }
                else {
                    (self.k[row_tag_id].f)( &mut self.cell, InputTagType::ColumnText( &mut self.col_row_pos_info.current_column_value,text.as_ref(), &mut sti ) );                                        
                }
                if self.can_change_range {
                    if was_ok_reading_range_rewrite.get() == true {
                        if let Some(new_dims) = sti.get_dimensions() {
                            self.reading_range = new_dims;
                            self.is_reading_range_defined = true;
                        }
                    }
                }
                self.drop_row_request = (*drop_it.get_mut()).clone();
            },
            false => {
            }
        }
    }
    pub fn run_text_user_tag_non_row<'c>(&mut self, row_tag_id: &[u8], e: &quick_xml::events::BytesText<'_>, f_r: &mut quick_xml::Reader<&'c [u8]>,dims: &mut Cell<radix10::Radix10Rectange> ) {
        match self.k.contains_key(row_tag_id) {
            true => {
                let text = e.unescape_and_decode(&f_r).unwrap();
                let was_ok_reading_range_rewrite: Cell<bool> = Cell::new(false);
           
                let info = NonRowInfo::new(&dims, self.is_reading_range_defined, &was_ok_reading_range_rewrite );
                (self.k[row_tag_id].f)( &mut self.cell, InputTagType::OtherTagText(text.as_ref(), info));                                        
                if self.can_change_range {
                    if was_ok_reading_range_rewrite.get() == true {
                        self.reading_range = dims.get();
                        self.is_reading_range_defined = true;      
                    }              
                }
                
            },
            false => {
            }
        }      
    }
    pub fn run_end_user_tag(&mut self, row_tag_id: &[u8], _: quick_xml::events::BytesEnd<'_>, drop_it: &mut Cell<Option<DropRowRequest>>) {
            match self.k.contains_key(row_tag_id) {
            true => {
            
            
                let dims: Cell<radix10::Radix10Rectange> = Cell::new(self.reading_range);
                let was_ok_reading_range_rewrite: Cell<bool> = Cell::new(false);
                let mut sti = StateInfo::new(CellInfoLocation::new(&self.col_row_pos_info, true, self.is_reading_range_defined, drop_it, &dims, &self.header_row, &was_ok_reading_range_rewrite ));                   
                                
                if self.row_tag == row_tag_id {
                    self.call_end_row_user_func = true;
                }
                else {
                    (self.k[row_tag_id].f)( &mut self.cell, InputTagType::ColumnEnd(&mut self.col_row_pos_info.current_column_value, &mut sti) );
                }
                if self.can_change_range {
                    if was_ok_reading_range_rewrite.get() == true {
                    
                        if let Some(new_dims) = sti.get_dimensions() {
                            self.reading_range = new_dims;
                            self.is_reading_range_defined = true;
                        }
                    }
                }
                self.drop_row_request = drop_it.get_mut().clone();
                
            },
            false => {
            }
        }
    }

    pub fn run_end_user_row_tag(&mut self, row_tag_id: &[u8], _: quick_xml::events::BytesEnd<'_>, drop_it: &mut Cell<Option<DropRowRequest>>, fin_rows: &mut super::row_validation::RowValidation<C> ) {
            match self.k.contains_key(row_tag_id) {
            true => {


                let dims: Cell<radix10::Radix10Rectange> = Cell::new(self.reading_range);
                let was_ok_reading_range_rewrite: Cell<bool> = Cell::new(false);
                let mut sti = StateInfo::new(CellInfoLocation::new(&self.col_row_pos_info, true, self.is_reading_range_defined, drop_it, &dims, &self.header_row, &was_ok_reading_range_rewrite ));                   
                
                if row_tag_id == self.row_tag {
                    (self.k[row_tag_id].f)( &mut self.cell, InputTagType::RowComplete( fin_rows) );
                }
                else {
                    (self.k[row_tag_id].f)( &mut self.cell, InputTagType::ColumnEnd(&mut self.col_row_pos_info.current_column_value, &mut sti) );
                }
                if self.can_change_range {
                    if was_ok_reading_range_rewrite.get() == true {
                    
                        if let Some(new_dims) = sti.get_dimensions() {
                            self.reading_range = new_dims;
                            self.is_reading_range_defined = true;
                        }
                    }
                }
                self.drop_row_request = drop_it.get_mut().clone();
                
            },
            false => {
            }
        }
    }

    fn hash_attribs_that_exist_both_user_and_xml(user_attbs: &Vec<helpers::RelativeNamePartsAttribs>, e: &quick_xml::events::BytesStart ) -> HashMap<String,String>{
        let mut hsh: HashMap<String,String> = HashMap::new();
        for expected_key in user_attbs {
            for attr in e.attributes() {
                match attr {
                    Ok( valid ) => {
                        
                        if expected_key.attrib == valid.key {
                            match String::from_utf8(valid.value.into_owned()) {
                                Ok( value_fine ) => {

                                    let vk = String::from_utf8_lossy(valid.key).into_owned();
                                    hsh.entry(vk).or_insert(value_fine);
                                },
                                Err( _ ) => {
                                    
                                }
                            }
                        }                    
                    },
                    Err( _ ) => {

                    }
                }                                      
            }                   
        }
        hsh
    }



    pub fn run_start_tags(&mut self, e: &quick_xml::events::BytesStart, just_getting_the_header: &mut bool, drop_it: &mut Cell<Option<DropRowRequest>>) -> Completion<C> {
        let this_tag = String::from_utf8_lossy(e.name()).into_owned();
        
        self.current_tag_line.modify_start_tag( this_tag );
        
        let zz = self.current_tag_line.clone();
        let vr = zz.tag_line.as_bytes();

        self.col_row_pos_info.is_sub_of_row = self.current_tag_line.tag_line.starts_with(self.row_tag_string.as_str());

        if self.col_row_pos_info.is_sub_of_row {

            if self.row_tag == vr {
                //new row so of course reset row column index
                self.call_end_row_user_func = false;
                self.col_row_pos_info.column_index = 0;
                self.col_row_pos_info.is_first_col = true;
                self.col_row_pos_info.current_column_header = None;
                self.col_row_pos_info.current_column_value = C::default();
                        

                //if gets to row tag and doesn't have a range then break None
                if !self.is_reading_range_defined {                            
                    return Completion::CriticalFail(CreateParserError::from("Range needs to be supplied before row/columns can be processed.") );
                }
                self.can_change_range = false;
                
                self.col_row_pos_info.last_row_index = self.col_row_pos_info.row_index;
                //how to handle row index
                match self.parse_flags {
                    f if f.contains(ParserFlags::ROWS_IN_SEQUENTIAL_ORDER_SKIP) && !f.contains(ParserFlags::ONLY_GET_ROWS_THAT_EXIST ) => {
                        if let Err( error ) = self.col_row_pos_info.update_row_index( &e, self.row_index_attribute_name ) {                                    
                            return Completion::RowFail(RowValidation::Invalid( error ) );
                        }
                        self.update_for_missing_rows();
                    },
                    f if !f.contains(ParserFlags::ROWS_IN_SEQUENTIAL_ORDER_SKIP) && f.contains(ParserFlags::ONLY_GET_ROWS_THAT_EXIST ) => {
                        if let Err( error ) = self.col_row_pos_info.update_row_index( &e, self.row_index_attribute_name ) {                                    
                            return Completion::RowFail( RowValidation::Invalid( error ));   
                        }
                    },
                    _ => {
                    
                    }
                }

                //Wanting the header to be returned?
                if self.drop_row_request.is_none() {                            
                    if self.parse_flags.contains(ParserFlags::HAS_HEADER) && self.reading_range.row == self.col_row_pos_info.row_index {
                        *just_getting_the_header = true;
                        
                    }
                }
                
                //Current row index is over reading range, if infinite then will hit the eof tag to end iterator.
                if self.is_row_greater_than_reading_range() {
                    return Completion::EndOfRows;
                }

                
            }

            if self.column_tag.is_some() {
                if self.column_tag.unwrap() == vr { //clear every column tag start in ref S
                    if self.drop_row_request.is_none() {
                        self.col_row_pos_info.current_column_value = C::default();
                        
                        if let Err( error ) = self.col_row_pos_info.update_col_index(&e, self.col_index_attribute_name) {                                    
                            return Completion::RowFail( RowValidation::Invalid( error ));
                        }
                        self.update_range_items();
                        self.col_row_pos_info.is_first_col = false;
                                                    
                        if !*just_getting_the_header {                     
                            self.col_row_pos_info.current_column_header = self.get_current_column_name_by_column_index();                 
                        }
                        else {
                            self.col_row_pos_info.current_column_header = None;
                        }
                    }
                }
            }
            
            self.update_range_items();

            let should_run_user_tag_handler = !*just_getting_the_header && self.drop_row_request.is_none() && (self.col_row_pos_info.in_range || !self.col_row_pos_info.is_sub_of_row);
            if should_run_user_tag_handler {
                let mut dims: Cell<radix10::Radix10Rectange> = Cell::new(radix10::Radix10Rectange::zero());
                self.run_start_user_tag(vr,e, drop_it, &mut dims);                        
            }
            else if *just_getting_the_header && self.drop_row_request.is_none() {
                if self.column_tag.is_some() {
                    if self.col_row_pos_info.in_range && self.column_tag.unwrap() == vr{
                        self.header_attributes = Self::get_all_tag_attributes(e);
                        
                    }
                }
            }
        }
        else {
          //  self.update_range_items();
            let should_run_user_tag_handler = !*just_getting_the_header && self.drop_row_request.is_none() && (self.col_row_pos_info.in_range || !self.col_row_pos_info.is_sub_of_row);
            if should_run_user_tag_handler {
                let mut dims: Cell<radix10::Radix10Rectange> = Cell::new(self.reading_range);
                
                self.run_start_user_tag_non_row(vr,e, &mut dims);                        
            }
        }

        Completion::Ok
    }
    pub fn run_end_tags(&mut self, e: quick_xml::events::BytesEnd, just_getting_the_header: &mut bool, drop_it: &mut Cell<Option<DropRowRequest>>, row_data: &mut Vec<(usize,C)>) -> Completion<C> {
        
        let rwq = self.current_tag_line.tag_line.clone();
        let vr = rwq.as_bytes();
            
        if self.col_row_pos_info.is_sub_of_row {

            let mut set_row_break = false;

            let should_run_user_tag_handler = !*just_getting_the_header && self.drop_row_request.is_none() && (self.col_row_pos_info.in_range || !self.col_row_pos_info.is_sub_of_row);
            if should_run_user_tag_handler {
                if self.row_tag == vr {
                    self.call_end_row_user_func = true;
                }
                else {
                    self.run_end_user_tag(vr,e.clone(), drop_it);                                          
                }
            }
            else if should_run_user_tag_handler && self.drop_row_request.is_none() {
                
            }

            if self.column_tag.is_some() {
                if self.column_tag.unwrap() == vr {
                    if self.drop_row_request.is_none() {
                                
                        if self.col_row_pos_info.in_range {
                            if !*just_getting_the_header {
                                row_data.push((self.col_row_pos_info.column_index,  self.col_row_pos_info.current_column_value.clone() ));
                            }
                            else {
                                if self.parse_flags.contains(ParserFlags::MODIFY_HEADER) {
                                    match self.k.contains_key(vr) {
                                        true => {
                                            
                                            let dims: Cell<radix10::Radix10Rectange> = Cell::new(radix10::Radix10Rectange::zero());
                                            let was_ok_reading_range_rewrite: Cell<bool> = Cell::new(false);            
                                            let mut sti = StateInfo::new(CellInfoLocation::new(&self.col_row_pos_info, true, self.is_reading_range_defined, drop_it, &dims, &self.header_row, &was_ok_reading_range_rewrite ));                   
                                            
                                            (self.k[vr].f)( &mut self.cell, InputTagType::ColumnHeader( &mut self.col_row_pos_info.current_column_value, &mut self.last_header_string, &mut self.header_attributes, &mut sti ) );
                                            
                                            if self.can_change_range {
                                                if was_ok_reading_range_rewrite.get() == true {
                
                                                    if let Some(new_dims) = sti.get_dimensions() {
                                                        self.reading_range = new_dims;
                                                    }
                                                }
                                            }
                                            self.drop_row_request = drop_it.get_mut().clone();
                                            
                                        },
                                        false => {
                                        }
                                    }         
                                    
                                }
                                self.header_row_vec.push(self.last_header_string.clone());
                                self.header_row.entry(self.last_header_string.clone()).or_insert(self.col_row_pos_info.column_index);
                            }
                            
                        }
                    }
                    
                }
            }
            
            if self.row_tag == vr {
                if self.col_row_pos_info.is_in_row_range {
                    if self.drop_row_request.is_none() {
                        if self.column_tag.is_none() {
                            row_data.push((self.col_row_pos_info.column_index, self.col_row_pos_info.current_column_value.clone() ));                                    
                        }
                    }
                    if self.drop_row_request.is_none() {
                        set_row_break = true;
                    }
                    else if let Some(DropRowRequest::Error(_)) = self.drop_row_request.clone() {
                        set_row_break = true;
                        drop_it.set(None);
                    }
                    else { //since we are just ignoring this value then have to set drop_it here so as to not drop following rows
                        drop_it.set(None);
                        
                        self.drop_row_request = None;
                    }
                } 
            }
        
            for _ in 0..self.current_tag_line.last_tag_sizes.pop().unwrap() {
                self.current_tag_line.tag_line.pop();
            }
            
            if set_row_break {

                        
                
                if !*just_getting_the_header {
                    if self.drop_row_request.is_none() {              
                        let mut data = self.sort_columns_for_return( row_data);
                        if self.row_tag == vr && self.call_end_row_user_func {                                    
                            self.run_end_user_row_tag(vr,e, drop_it,&mut data);
                        }
                        return Completion::OkWithRow(data);
                    }
                    else {
                        if let Some(DropRowRequest::Error(errz)) = self.drop_row_request.clone() {
                            let mut data = RowValidation::Invalid(CreateParserError::from(errz.as_str()));
                            if self.row_tag == vr && self.call_end_row_user_func  {
                                self.run_end_user_row_tag(vr,e, drop_it,&mut data);
                            }
                            return Completion::OkWithRow(data);
                            
                        }
                    }     
                }
                else {
                    self.col_row_pos_info.is_first_row = false;
                    *just_getting_the_header = false;
                    if self.parse_flags.contains(ParserFlags::RETURN_HEADER) {
                        let data = RowValidation::Header( self.header_row.clone() );
                        
                        if self.row_tag == vr {
                            if self.k.contains_key(vr) {

                                let mut c_err: HeaderCompleteHandler = HeaderCompleteHandler{ reason: String::new(), cancel: false };
                                (self.k[vr].f)( &mut self.cell, InputTagType::HeaderComplete(self.header_row.clone(), &mut c_err) );
                                if c_err.cancel {                                    
                                    return Completion::CriticalFail(CreateParserError::from( format!("{}", c_err.reason).as_ref() ));
                                }
                            }
                        }
            
                        return Completion::OkWithRow(data);
                    }
                }   
            }
        }
        else {
            for _ in 0..self.current_tag_line.last_tag_sizes.pop().unwrap() {
                self.current_tag_line.tag_line.pop();
            }
            let should_run_user_tag_handler = !*just_getting_the_header && self.drop_row_request.is_none() && (self.col_row_pos_info.in_range || !self.col_row_pos_info.is_sub_of_row);
            if should_run_user_tag_handler {
                let mut dims: Cell<radix10::Radix10Rectange> = Cell::new(self.reading_range);
                
                self.run_end_user_tag_non_row(vr,e, &mut dims);                        
            }
        }

        Completion::Ok
    }

    pub fn run_text_tags<'c>(&mut self, e: &quick_xml::events::BytesText, just_getting_the_header: &mut bool, drop_it: &mut Cell<Option<DropRowRequest>>,f_r: &mut quick_xml::Reader<&'c [u8]>) -> Completion<C> {
        
        let rwq = self.current_tag_line.tag_line.clone();
        let vr = rwq.as_bytes();
            
        if self.col_row_pos_info.is_sub_of_row {
            let should_run_user_tag_handler = !*just_getting_the_header && self.drop_row_request.is_none() && (self.col_row_pos_info.in_range || !self.col_row_pos_info.is_sub_of_row);
            if should_run_user_tag_handler {                        

                let zz = self.current_tag_line.clone();
                let vr = zz.tag_line.as_bytes();                        
                self.run_text_user_tag(vr,e,drop_it, f_r);
            }
            else if *just_getting_the_header && self.drop_row_request.is_none() {//&& self.parser_data.user_manage_headers {
                let text = e.unescape_and_decode(&f_r).unwrap();
                self.last_header_string = text;         
            }
        }
        else {
            let should_run_user_tag_handler = !*just_getting_the_header && self.drop_row_request.is_none() && (self.col_row_pos_info.in_range || !self.col_row_pos_info.is_sub_of_row);
            if should_run_user_tag_handler {
                let mut dims: Cell<radix10::Radix10Rectange> = Cell::new(self.reading_range);
                
                self.run_text_user_tag_non_row(vr,e, f_r, &mut dims);                        
            }
            
        }
        Completion::Ok
     }
     fn get_all_tag_attributes(e: &quick_xml::events::BytesStart ) -> HashMap<String,String>{
        let mut hsh: HashMap<String,String> = HashMap::new();
        for attr in e.attributes() {
            match attr {
                Ok( valid ) => {
                    
                    match String::from_utf8(valid.value.into_owned()) {
                        Ok( value_fine ) => {

                            let vk = String::from_utf8_lossy(valid.key).into_owned();
                            hsh.entry(vk).or_insert(value_fine);
                        },
                        Err( _ ) => {
                            
                        }
                    }
                                    
                },
                Err( _ ) => {

                }
            }                                      
        }                   
        
        hsh
    }


    fn sort_columns_for_return(&self, row_data: &mut Vec<(usize,C)>) -> RowValidation<C> {
        let mut ve = vec![C::default(); self.reading_range.column_count];                           
        let mut k = row_data.to_owned();
        k.sort_by(|a,b| a.0.cmp(&b.0));
        for dta in k {
            let d = dta.0 - self.reading_range.column;
            ve[d] = dta.1;
        }

        RowValidation::Valid(ve)
    }
}