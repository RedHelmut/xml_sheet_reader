
pub struct HeaderCompleteHandler {
    pub(crate) cancel: bool,
    pub(crate) reason: String,
}
impl HeaderCompleteHandler {
    pub fn cancel_operation( &mut self, reason: String ) {
        self.cancel = true;
        self.reason.clone_from(&reason);
    }
}