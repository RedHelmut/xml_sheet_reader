use bitflags;

bitflags! {
    pub struct ParserFlags: u32 {
        const MODIFY_HEADER= 0b00000001;
        const RETURN_HEADER = 0b00000010;
        const ROWS_IN_SEQUENTIAL_ORDER_SKIP = 0b00000100;
        const HAS_HEADER = 0b00010000;
        const ONLY_GET_ROWS_THAT_EXIST = 0b00100000;
        const DEFAULT_FLAGS_WITH_HEADER = ParserFlags::ROWS_IN_SEQUENTIAL_ORDER_SKIP.bits | ParserFlags::HAS_HEADER.bits;
        const DEFAULT_FLAGS_NO_HEADER = ParserFlags::ROWS_IN_SEQUENTIAL_ORDER_SKIP.bits;        
        const FULL_HEADER = ParserFlags::ROWS_IN_SEQUENTIAL_ORDER_SKIP.bits | ParserFlags::HAS_HEADER.bits | ParserFlags::MODIFY_HEADER.bits | ParserFlags::RETURN_HEADER.bits;
        
    }
}
