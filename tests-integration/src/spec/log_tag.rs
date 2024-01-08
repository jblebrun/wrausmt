#[derive(Debug)]
pub enum Tag {
    Spec,
    SpecModule,
}
impl wrausmt_common::logger::Tag for Tag {
    fn enabled(&self) -> bool {
        matches!(self, Tag::Spec)
    }
}
