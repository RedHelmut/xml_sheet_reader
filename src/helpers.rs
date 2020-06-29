
#[derive(Clone)]
pub enum RelativeNamePartsAttribsProps {
    Capture,
    NonCapture,
}

#[derive(Clone)]
pub struct RelativeNamePartsAttribs<'b> {
    pub attrib: &'b [u8],
    pub capture: RelativeNamePartsAttribsProps,
}

#[derive(Clone)]
pub struct RelativeNameParts<'b> {
    pub name: &'b [u8],
    pub attribs: Vec<RelativeNamePartsAttribs<'b>>
}


//Parse attribs, allows for () with no implementation.
fn parse_relative_name_parts_attribs<'a>( attr: Vec<&'a [u8]>) -> Vec<RelativeNamePartsAttribs> {
    
    let mut attb: Vec<RelativeNamePartsAttribs> = Vec::new();
    for i in attr {
        let mut start_par = 0;
        let mut par_start_count = 0;
        let mut found_start_par = false;

        let mut end_par = 0;
        let mut par_end_count = 0;
        let mut found_end_par = false;
        for r in 0..i.len() {
            let c: char = i[r] as char;
            if c == '(' {
                start_par = r;
                found_start_par = true;
                par_start_count = par_start_count + 1;
            }

            if c == ')' {
                end_par = r;
                found_end_par = true;
                par_end_count = par_end_count + 1;
            }
            
        }
        match (found_start_par, found_end_par, par_start_count, par_end_count, start_par < end_par) {
            (true,true,1,1,true) => {                
                if end_par - start_par > 0 {
                    attb.push(RelativeNamePartsAttribs { attrib:  &i[start_par + 1..end_par], capture: RelativeNamePartsAttribsProps::Capture});
                }
                else {
                }
                
            },
            
            (_,_,_,_,_) => {
                  attb.push(RelativeNamePartsAttribs { attrib:  &i, capture: RelativeNamePartsAttribsProps::NonCapture});
            }
        };
    }
    attb

}

///Used to turn str into RelativeNameParts. 
///Example str "tag1/tag2/tag3|attrib1,attrib2|" into name tag1/tag2/tag3 and attribs attrib1, attrib2 that allow for inf row length
pub fn get_relative_name_parts<'a>( s: &'a[u8] ) -> RelativeNameParts {
    let split_off_attributes = s.split(|x| x == &b'|').collect::<Vec<&[u8]>>();
    //No attributes, or sloppy configuration and ignore

    let attribbs:Vec<&'a [u8]> = 
        match split_off_attributes.len() != 3 {
            true => {
                Vec::new()
            },
            false => {
                split_off_attributes[1].split(|x| x == &b',' ).collect::<Vec<_>>()
                
            }
        
        };
    RelativeNameParts {
        name: split_off_attributes[0],
        attribs: parse_relative_name_parts_attribs(attribbs)
    }
}


