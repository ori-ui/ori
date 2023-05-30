use std::fmt::Display;

use ori_graphics::Rect;

use crate::StyleSelectors;

/// Debug information about the current state of the UI.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Debug {
    /// The root element of the UI.
    pub tree: DebugElement,
}

impl Display for Debug {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.tree)
    }
}

/// Debug information about a single element.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DebugElement {
    /// The selectors of the element.
    pub selectors: StyleSelectors,
    /// The local rect of the element.
    pub local_rect: Rect,
    /// The global rect of the element.
    pub global_rect: Rect,
    /// The children of the element.
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
