
use std::str;
use std::fmt;
pub struct Radix10Error {
    pub message: String
}
impl fmt::Display for Radix10Error {
    fn fmt(&self, f: &mut fmt::Formatter ) -> fmt::Result {
        write!(f, "Error in Radix10 {} " , self.message)
    }
}
impl fmt::Debug for Radix10Error {
    fn fmt(&self, f: &mut fmt::Formatter ) -> fmt::Result {
        write!(f, "Error in Radix10 {}", self.message)
    }
}
impl From<&str> for Radix10Error {
    fn from(inp: &str) -> Radix10Error {
        Radix10Error {
            message: inp.to_owned()
        }
    }
}
impl std::error::Error for Radix10Error {
    fn description(&self) -> &str {
        "Parse error"
    }
}
pub enum NumberType {
    Infinite(usize),
    Real((usize,usize)),
}

#[derive(Default,Debug, Clone, Copy)]
pub struct Radix10Rectange {
    pub column: usize,
    pub row: usize,
    pub column_count: usize,
    pub row_count: usize,
    pub is_row_count_infinite: bool,
}
impl Radix10Rectange {

 pub fn new( cell_ref: &str ) -> Result<Self, Radix10Error> {
     if cell_ref.trim().len() == 0 {
         return Err(Radix10Error::from("Canno't be an empty string"));
     }
        match Radix10Rectange::get_row_col_real_index( cell_ref ) {
            (Ok( NumberType::Real( p_top_left)), Ok(NumberType::Real( p_bottom_right)) ) => {

                let (col, col_count) = 
                    match p_top_left.0 > p_bottom_right.0 {
                        true => {
                            (p_bottom_right.0, p_top_left.0 - p_bottom_right.0)
                        },
                        false => {
                            (p_top_left.0, p_bottom_right.0 - p_top_left.0)
                        }
                    };
                let (row,row_count) = 
                    match p_top_left.1 > p_bottom_right.1 {
                        true => {
                            (p_bottom_right.1, p_top_left.1 - p_bottom_right.1)  
                        },
                        false => {
                            (p_top_left.1, p_bottom_right.1 - p_top_left.1 )
                        }
                    };

                Ok(Radix10Rectange {
                    column: col,
                    row: row,
                    column_count: col_count + 1,
                    row_count: row_count + 1,
                    is_row_count_infinite: false, 
                })

            },
            (Ok( NumberType::Real( p_top_left)), Ok(NumberType::Infinite( p_bottom_right)) ) => {
 
                let (col, col_count) = 
                    match p_top_left.0 > p_bottom_right {
                        true => {
                            (p_bottom_right, p_top_left.0 - p_bottom_right)
                        },
                        false => {
                            (p_top_left.0, p_bottom_right - p_top_left.0)
                        }
                    };          

                Ok(Radix10Rectange {
                    column: col,
                    row: p_top_left.1,
                    column_count: col_count + 1,
                    row_count: 0 + 1,
                    is_row_count_infinite: true, 
                })

                
            },
            (Ok( NumberType::Infinite( inf_column )), Ok(NumberType::Real( (col_ind,bottom_row_ind))))  => {

                let (col, col_count) = 
                    match inf_column > col_ind {
                        true => {
                            (col_ind, inf_column - col_ind)
                        },
                        false => {
                            (inf_column, col_ind - inf_column)
                        }
                    };          

                Ok(Radix10Rectange {
                    column: col,
                    row: bottom_row_ind,
                    column_count: col_count + 1,
                    row_count: 0,
                    is_row_count_infinite: true, 
                })

  
            },
            (Ok( NumberType::Infinite( inf_column )), Ok(NumberType::Infinite( inf_column_two)))  => {

                let (col, col_count) = 
                    match inf_column > inf_column_two {
                        true => {
                            (inf_column_two, inf_column - inf_column_two)
                        },
                        false => {
                            (inf_column, inf_column_two - inf_column)
                        }
                    };          

                Ok(Radix10Rectange {
                    column: col,
                    row: 0,
                    column_count: col_count + 1,
                    row_count: 0,
                    is_row_count_infinite: true, 
                })

  
            },
            (Err(error1), Err(_)) => {
                Err( error1 )
            },            
            (_,_) => {
                Err( Radix10Error::from("Undescripbed error"))
            }

        }
    
    }    
    pub fn zero() -> Self {
        Radix10Rectange {
            column: 0,
            row: 0,
            column_count: 0,
            row_count: 0,
            is_row_count_infinite: false,
        }
    }
  
    pub fn get_real_index( input_str: &str ) -> Result<NumberType, Radix10Error> {

        if input_str.len() == 0 {
            return Err(Radix10Error::from("Input is empty") );
        }

        let mut column = 0;
        let mut row = 0;
        let mut last_pow = 1;
        let mut is_inf = false;
        let mut row_arr = Vec::new();
        let mut col_arr = Vec::new();
        let mut is_num = false;
        for (_, val) in input_str.char_indices() {        
            match val.is_ascii_alphabetic() && val.is_ascii_uppercase() && !is_num {
                true => {    
                    match val.to_digit(36) { //A is 10
                        Some( num ) => {
                            col_arr.push(num);                        
                        },
                        None =>
                        {
                            return Err(Radix10Error::from("Input contains invalid digit.") );
                        }
                    }
                },
                false => {//out of the Column type now to the number
                    is_num = true;                        
                    if val == '*' {
                        is_inf = true;
                    }
                    else {
                        if is_inf { //error if it is
                            return Err(Radix10Error::from("If infinite then only have the * symbol and nothing else after") );
                        }
                        else {
                            match val.to_digit(10) {
                                Some( num ) => {
                                    row_arr.push(num);
                                },
                                None => {
                                    return Err(Radix10Error::from("Input contains invalid digit.") );
                                }
                            }
                        }
                    }
                }
            }
        }
        col_arr.reverse();
        for num in col_arr {        
            let alphabet_place = num - 9;        
            column = column + last_pow * alphabet_place as usize;
            last_pow = 26 * last_pow;
        }
        last_pow = 1;
        if !is_inf {
            row_arr.reverse();
            for num in row_arr {             
                row = row + last_pow * num as usize;
                last_pow = 10 * last_pow;
            }
        }
        if column == 0 || (row == 0 && !is_inf) {
            Err(Radix10Error::from("Not a valid xlsx ref."))
        }
        else if is_inf {
            Ok(NumberType::Infinite(column - 1) )
        }
        else {
            Ok( NumberType::Real((column - 1, row - 1)) )
        }
    }
        
    pub fn get_row_col_real_index(cell_ref: &str ) -> (Result<NumberType,Radix10Error>,Result<NumberType,Radix10Error>) {

        let dims = cell_ref.split(":");
        let data = dims.collect::<Vec<_>>();
        match data.len() {
            cnt if cnt == 1 => {
                //apparently a single cell
                (Err(Radix10Error::from("Need to have a : to get column and row")), Err(Radix10Error::from("Need to have a : to get column and row")))
            }
            cnt if cnt == 2 => {
                let start = data[0];
                let end = data[1];

                let start_ind = Radix10Rectange::get_real_index(start);
                let end_ind = Radix10Rectange::get_real_index(end);
                ((start_ind),(end_ind))                        
            },
            _ => {
                (Err(Radix10Error::from("Too many : in input")), Err(Radix10Error::from("Too many : in input")))
            }

        }
  
    }
}
