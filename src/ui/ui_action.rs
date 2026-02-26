#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UiAction {
    None,
    Quit,
    StartSession,
    NextStep,
    PrevStep,
    JumpRecommended,
    JumpBack,
    Verify,
    Hint,
    Suggest(Option<usize>),
    ClearLog,
    ShowHelp,
    DismissPopup,
    RunCommand(String),
    ForceRunCommand(String),
}
