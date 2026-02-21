/// UI language selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiLang {
    Zh,
    En,
}

/// Filter for the provider list view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderFilter {
    All,
    China,
    USA,
    Global,
}

/// Top-level navigation view.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActiveView {
    Task,
    Api,
    Network,
    Ollama,
    Resources,
}
