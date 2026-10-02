//! Central editor-command metadata. Menus, toolbar buttons, the compact tool rail,
//! keyboard handling and tooltips must all describe the same command contract.

use crate::CanvasTool;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StudioCommand {
    Select,
    Paint,
    Erase,
    Pick,
    SaveScene,
    Undo,
    Redo,
    TogglePlay,
    ToggleLauncher,
    DeleteSelection,
    FocusCanvas,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CommandSpec {
    pub id: &'static str,
    pub label: &'static str,
    pub shortcut: &'static str,
    pub description: &'static str,
}

pub fn spec(command: StudioCommand) -> CommandSpec {
    match command {
        StudioCommand::Select => CommandSpec {
            id: "havenwild.canvas.select",
            label: "Select",
            shortcut: "Q",
            description: "Select and manipulate the authored object or source-bound visual under the cursor.",
        },
        StudioCommand::Paint => CommandSpec {
            id: "havenwild.canvas.paint",
            label: "Paint",
            shortcut: "B",
            description: "Place the currently selected exact source region on the active visual layer.",
        },
        StudioCommand::Erase => CommandSpec {
            id: "havenwild.canvas.erase",
            label: "Erase",
            shortcut: "E",
            description: "Remove the selected authored object or clear the active visual-layer cell without modifying source art.",
        },
        StudioCommand::Pick => CommandSpec {
            id: "havenwild.canvas.pick",
            label: "Pick",
            shortcut: "P",
            description: "Sample the exact original source region already used at the hovered canvas location.",
        },
        StudioCommand::SaveScene => CommandSpec {
            id: "havenwild.scene.save",
            label: "Save Scene",
            shortcut: "Ctrl+S",
            description: "Save the current authored Summer scene draft atomically.",
        },
        StudioCommand::Undo => CommandSpec {
            id: "havenwild.edit.undo",
            label: "Undo",
            shortcut: "Ctrl+Z",
            description: "Undo the most recent authored scene transaction.",
        },
        StudioCommand::Redo => CommandSpec {
            id: "havenwild.edit.redo",
            label: "Redo",
            shortcut: "Ctrl+Y",
            description: "Redo the most recently undone authored scene transaction.",
        },
        StudioCommand::TogglePlay => CommandSpec {
            id: "havenwild.pie.toggle",
            label: "Play Scene",
            shortcut: "F6",
            description: "Toggle play-in-editor using a snapshot of the same authored scene without mutating source files.",
        },
        StudioCommand::ToggleLauncher => CommandSpec {
            id: "havenwild.apps.launcher",
            label: "Havenwild Tools",
            shortcut: "Ctrl+Space",
            description: "Open or close the bottom-left Havenwild tool-application launcher.",
        },
        StudioCommand::DeleteSelection => CommandSpec {
            id: "havenwild.edit.delete",
            label: "Delete Selection",
            shortcut: "Delete",
            description: "Remove the currently selected authored object or visual-layer cell.",
        },
        StudioCommand::FocusCanvas => CommandSpec {
            id: "havenwild.canvas.focus",
            label: "Focus Canvas",
            shortcut: "F11",
            description: "Temporarily hide editor chrome and focus the world canvas.",
        },
    }
}

pub fn tool_command(tool: CanvasTool) -> StudioCommand {
    match tool {
        CanvasTool::Select => StudioCommand::Select,
        CanvasTool::Paint => StudioCommand::Paint,
        CanvasTool::Erase => StudioCommand::Erase,
        CanvasTool::Sample => StudioCommand::Pick,
    }
}

pub fn tooltip(command: StudioCommand) -> String {
    let item = spec(command);
    format!("{}\n\nShortcut: {}", item.description, item.shortcut)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;

    #[test]
    fn direct_canvas_tools_have_unique_documented_shortcuts() {
        let tools = [
            CanvasTool::Select,
            CanvasTool::Paint,
            CanvasTool::Erase,
            CanvasTool::Sample,
        ];
        let mut shortcuts = BTreeSet::new();
        for tool in tools {
            let item = spec(tool_command(tool));
            assert!(!item.id.is_empty());
            assert!(!item.shortcut.is_empty());
            assert!(tooltip(tool_command(tool)).contains(item.shortcut));
            assert!(shortcuts.insert(item.shortcut));
        }
    }
}
