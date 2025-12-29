use std::collections::BTreeMap;

use kdl::{KdlDocument, KdlNode};
use tmux::{LayoutNode, Preset, SplitDirection, Window};

pub fn parse_config(doc_str: &str) -> Result<BTreeMap<String, Preset>, String> {
    let doc: KdlDocument = doc_str.parse().unwrap();

    let nodes: &[KdlNode] = doc.nodes();

    let mut map = BTreeMap::<String, Preset>::new();

    // nodes.iter().map(|node| parse_session(node)).collect()
    for node in nodes.iter().map(|node| parse_session(node)) {
        let node = node?;
        map.insert(node.name.clone(), node);
    }
    return Ok(map)
}

fn parse_session(session: &KdlNode) -> Result<Preset, String> {
    if session.name().value() != "session" {
        return Err("Node is not a session".to_string());
    }

    let session_name: &str = session
        .get("name")
        .map(|name| name.as_string())
        .flatten()
        .ok_or("Missing or invalid session name!")?;

    let session_cwd: &str = session
        .get("cwd")
        .map(|name| name.as_string())
        .flatten()
        .unwrap_or("~");

    let windows: Vec<Window> = match session.children() {
        Some(session_children) => parse_windows(session_children.nodes(), session_cwd)?,

        // If the session does not specify any windows, assume single window with single pane that
        // inherits cwd from session
        None => vec![Window {
            name: "main".to_string(),
            cwd: session_cwd.to_string(),
            layout: LayoutNode::Pane {
                cwd: session_cwd.to_string(),
                command: None,
                size: 100,
            },
        }],
    };

    Ok(Preset {
        name: session_name.to_string(),
        cwd: session_cwd.to_string(),
        windows,
        running: false,
    })
}

fn parse_windows(windows: &[KdlNode], parent_cwd: &str) -> Result<Vec<Window>, String> {
    if windows.is_empty() {
        return Ok(vec![Window {
            name: "main".to_string(),
            cwd: parent_cwd.to_string(),
            layout: LayoutNode::Pane {
                cwd: parent_cwd.to_string(),
                command: None,
                size: 100,
            },
        }]);
    }

    let mut ret = vec![];
    for (idx, window) in windows.iter().enumerate() {
        let node_name = window.name().value();
        if node_name != "window" {
            return Err(format!("Unknown session child node: `{node_name}`"));
        } else {
            // Extract window properties
            // ex: window name="bobby" cwd="~/bobby/" { ... }
            let window_cwd = window
                .get("cwd")
                .map(|cwd| cwd.as_string())
                .flatten()
                .unwrap_or(parent_cwd);
            let idx_str = idx.to_string();
            let window_name = window
                .get("name")
                .map(|cwd| cwd.as_string())
                .flatten()
                .unwrap_or(&idx_str.as_str());

            let panes: LayoutNode = match window.children() {
                Some(window_children) => parse_panes(window_children.nodes(), window_cwd)?,
                None => LayoutNode::Pane {
                    cwd: window_cwd.to_string(),
                    command: None,
                    size: 100,
                },
            };

            ret.push(Window {
                name: window_name.to_string(),
                cwd: window_cwd.to_string(),
                layout: panes,
            });
        }
    }
    // make a vec of windows, push each iterations generated value to it, return that value
    if ret.is_empty() {
        ret.push(Window {
            name: "name".to_string(),
            cwd: parent_cwd.to_string(),
            layout: LayoutNode::Pane {
                cwd: parent_cwd.to_string(),
                command: None,
                size: 100,
            },
        });
    }
    Ok(ret)
}

fn parse_panes(window_children: &[KdlNode], window_cwd: &str) -> Result<LayoutNode, String> {
    if window_children.is_empty() {
        return Ok(LayoutNode::Pane {
            cwd: window_cwd.to_string(),
            command: None,
            size: 100,
        });
    }

    if window_children.len() != 1 {
        return Err("Expected exactly one root `split` or `pane` node".into());
    }

    // The root node of a window should always occupy 100%
    let mut root_node = parse_node_recursive(&window_children[0], window_cwd)?;
    set_size(&mut root_node, 100);
    Ok(root_node)
}

fn parse_node_recursive(node: &KdlNode, parent_cwd: &str) -> Result<LayoutNode, String> {
    let node_name = node.name().value();

    // We try to get the size, but keep it as Option to know if it was omitted
    let explicit_size = node
        .get("size")
        .and_then(|v| v.as_integer())
        .map(|v| v as u8);

    match node_name {
        "pane" => {
            let cwd = node
                .get("cwd")
                .and_then(|v| v.as_string())
                .unwrap_or(parent_cwd)
                .to_string();

            let command = node
                .get("command")
                .and_then(|v| v.as_string())
                .map(|s| s.to_string());

            Ok(LayoutNode::Pane {
                cwd,
                command,
                size: explicit_size.unwrap_or(0), // Placeholder
            })
        }
        "split" => {
            let dir_str = node
                .get("direction")
                .and_then(|v| v.as_string())
                .unwrap_or("v");

            let direction = match dir_str {
                "h" | "horizontal" => SplitDirection::Horizontal,
                "v" | "vertical" => SplitDirection::Vertical,
                _ => return Err(format!("Invalid direction: `{}`", dir_str)),
            };

            let mut children = Vec::new();
            let mut total_explicit = 0u8;
            let mut missing_indices = Vec::new();

            if let Some(document) = node.children() {
                for (i, child_node) in document.nodes().iter().enumerate() {
                    let mut layout_child = parse_node_recursive(child_node, parent_cwd)?;

                    // Check if this specific child had a size defined
                    if let Some(p) = child_node.get("size").and_then(|v| v.as_integer()) {
                        let p = p as u8;
                        set_size(&mut layout_child, p);
                        total_explicit += p;
                    } else {
                        missing_indices.push(i);
                    }
                    children.push(layout_child);
                }
            }

            if children.is_empty() {
                return Err("Split nodes must contain children".into());
            }

            // --- Equal Distribution Logic ---
            if !missing_indices.is_empty() {
                let remaining = if total_explicit >= 100 {
                    0
                } else {
                    100 - total_explicit
                };
                let share = remaining / (missing_indices.len() as u8);

                for idx in missing_indices {
                    set_size(&mut children[idx], share);
                }
            }

            Ok(LayoutNode::Split {
                direction,
                children,
                size: explicit_size.unwrap_or(0), // Placeholder
            })
        }
        x => Err(format!("Unexpected node: `{x}`")),
    }
}

// Helper to set size regardless of enum variant
fn set_size(node: &mut LayoutNode, val: u8) {
    match node {
        LayoutNode::Pane { size, .. } => *size = val,
        LayoutNode::Split { size, .. } => *size = val,
    }
}

#[cfg(test)]
mod tests {
    use crate::parse_config;

    #[test]
    fn test_example() {
        let doc_str: String = std::fs::read("examples/config.kdl")
            .unwrap()
            .try_into()
            .unwrap();

        let presets = parse_config(&doc_str).unwrap();
        println!("{:?}", presets);
    }
}
