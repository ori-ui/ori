use crate::StyleSelectors;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Debug {
    pub tree: DebugNode,
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DebugNode {
    pub selectors: StyleSelectors,
    pub children: Vec<DebugNode>,
}
