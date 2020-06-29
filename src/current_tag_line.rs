
#[derive(Clone)]
pub struct CurrentTagLine {
    pub tag_line: String,
    pub last_tag_sizes: Vec<usize>,
}
///Helper function that takes the current tag and puts it like so prevprevtag/prevtag/tag
///Modifies a ParserData struct, no return.
impl CurrentTagLine {
    pub fn modify_start_tag<'a, 'b>(&'a mut self, this_tag: String){
        //refz.resize(this_tag.len(), 0);
        let mut tag_length = this_tag.len();
        if self.tag_line != String::new() {
            self.tag_line.push('/');
            tag_length = tag_length + 1;
        }                   
        self.last_tag_sizes.push(tag_length);                   
        self.tag_line.push_str(this_tag.as_ref());                    
    }
}
