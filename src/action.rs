#[derive(PartialEq, Debug)]
pub enum Action {
    Report,
    Filter,
    Add,
    Annotate,
    Subprocess,
    Log,
    Modify,
    HelpPopup,
    Error,
    ContextMenu,
    Jump,
    DeletePrompt,
    DonePrompt,
}
