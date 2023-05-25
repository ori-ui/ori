use std::fmt::Display;

use ori_graphics::Rect;

use crate::StyleSelectors;

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Debug {
    pub tree: DebugElement,
}

impl Display for Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tree)
    }
}

#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DebugElement {
    pub selectors: StyleSelectors,
    pub local_rect: Rect,
    pub global_rect: Rect,
    pub children: Vec<DebugElement>,
}

fn debug_recursive(
    f: &mut std::fmt::Formatter<'_>,
    element: &DebugElement,
    depth: usize,
) -> std::fmt::Result {
    let Some(selector) = element.selectors.last() else {
        for child in &element.children {
            write!(f, "{}", child)?;
        }

        return Ok(());
    };

    let indent = " ".repeat(depth * 2);

    if element.children.is_empty() {
        writeln!(f, "{}<{}/>", indent, selector)?;
    } else {
        writeln!(f, "{}<{}>", indent, selector)?;

        for child in &element.children {
            debug_recursive(f, child, depth + 1)?;
        }

        writeln!(f, "{}</{}>", indent, selector)?;
    }

    Ok(())
}

impl Display for DebugElement {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        debug_recursive(f, self, 0)
    }
}
