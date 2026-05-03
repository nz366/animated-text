#[derive(PartialEq, Clone, Copy)]
pub enum EditMode {
    Time,
    Progress,
}

#[derive(PartialEq, Clone, Copy)]
pub enum ViewMode {
    Line,
    List,
    TextEdit,
    DraftSelector,
}
