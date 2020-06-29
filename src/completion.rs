use super::RowValidation;
use super::CreateParserError;

pub enum Completion<C>
where C: Clone + Default + ToString {
    RowFail( RowValidation<C> ),
    EndOfRows,
    Ok,
    CriticalFail( CreateParserError ),
    OkWithRow(RowValidation<C>),
}