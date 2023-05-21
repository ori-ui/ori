use std::fmt::Display;

use ori_graphics::Rect;

use crate::StyleSelectors;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Debug {
    pub tree: DebugNode,
}

impl Display for Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tree)
    }
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DebugNode {
    pub selectors: StyleSelectors,
    pub local_rect: Rect,
    pub global_rect: Rect,
    pub children: Vec<DebugNode>,
}

fn debug_recursive(
    f: &mut std::fmt::Formatter<'_>,
    node: &DebugNode,
    depth: usize,
) -> std::fmt::Result {
    let Some(selector) = node.selectors.last() else {
        for child in &node.children {
            write!(f, "{}", child)?;
        }

        return Ok(());
    };

    let indent = " ".repeat(depth * 2);

    if node.children.is_empty() {
        writeln!(f, "{}<{}/>", indent, selector)?;
    } else {
        writeln!(f, "{}<{}>", indent, selector)?;

        for child in &node.children {
            debug_recursive(f, child, depth + 1)?;
        }

        writeln!(f, "{}</{}>", indent, selector)?;
    }

    Ok(())
}

impl Display for DebugNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug_recursive(f, self, 0)
    }
}
