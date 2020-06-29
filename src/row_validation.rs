use super::create_parser_error::CreateParserError;
use std::collections::HashMap;
///This is the return type of the CreateParser next function

#[derive(Clone)]
pub enum RowValidation<C>
where C: Clone + Default + ToString  {
    Valid(Vec<C>),
    Invalid(CreateParserError),
    Header( HashMap<String, usize> ),
    CriticalFail( CreateParserError ),
}
impl<C> RowValidation<C> 
where C: Clone + Default + ToString {
    pub fn is_valid( &self ) -> bool {
        match self {
            RowValidation::Valid(_) => {
                true
            },
            _ => {
                false
            }
        }
    }
    pub fn valid( self ) -> Option<Self> {
        match self {
            RowValidation::Valid(_) => {
                Some(self)
            },
            _ => {
                None
            }
        }
    }
    pub fn is_invalid( &self ) -> bool {
        match self {
            RowValidation::Invalid(_) => {
                true
            },
            _ => {
                false
            }
        }
    }
    pub fn invalid( self ) -> Option<Self> {
        match self {
            RowValidation::Invalid(_) => {
                Some(self)
            },
            _ => {
                None
            }
        }
    }    
    pub fn get_invalid( self ) -> Option<CreateParserError> {
        if let RowValidation::Invalid(x) = self {
            Some(x)
        }
        else {
            None
        }                
    }
    
   
}
