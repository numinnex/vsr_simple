#[derive(Debug, Default, PartialEq)]
pub enum Status {
    #[default]
    Normal,
    Recovery,
    ViewChange,
}
