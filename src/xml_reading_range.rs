pub enum XmlReadingRange<'a> {
    Defined( &'a str ),
    WillDefineBeforeRows,
}