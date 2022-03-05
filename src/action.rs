#[derive(Clone, PartialEq, Debug, Copy)]
pub enum Action {
    Report,
    Filter,
    Add,
    Annotate,
    Subprocess,
    Log,
    Modify,
    HelpPopup,
    ContextMenu,
    Jump,
    DeletePrompt,
    DonePrompt,
    Error,
}
