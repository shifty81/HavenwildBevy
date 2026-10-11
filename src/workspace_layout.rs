//! M2D131A - the serializable workspace docking authority.
//!
//! The Game Canvas is intentionally not part of this graph: it is the permanent
//! central host. The dock graph stores document-independent *workstation UI*
//! state only. Pixel/asset/scene state must never be serialized here.
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

pub const WORKSPACE_SCHEMA: &str = "havenwild.bevy.workspace_layout.v1";

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DockPanel {
    PixelStudio,
    GameTools,
    SceneNavigator,
    Inspector,
    AssetBrowser,
    Console,
    Problems,
    Jobs,
    Debug,
    AnimationTimeline,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SplitAxis {
    Horizontal,
    Vertical,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DockNode {
    Tabs {
        tabs: Vec<DockPanel>,
        active: DockPanel,
    },
    Split {
        axis: SplitAxis,
        ratio: f32,
        first: Box<DockNode>,
        second: Box<DockNode>,
    },
}
impl DockNode {
    fn single(panel: DockPanel) -> Self {
        Self::Tabs {
            tabs: vec![panel],
            active: panel,
        }
    }
    fn contains(&self, panel: DockPanel) -> bool {
        match self {
            Self::Tabs { tabs, .. } => tabs.contains(&panel),
            Self::Split { first, second, .. } => first.contains(panel) || second.contains(panel),
        }
    }
    fn collect(&self, all: &mut Vec<DockPanel>) {
        match self {
            Self::Tabs { tabs, .. } => all.extend(tabs),
            Self::Split { first, second, .. } => {
                first.collect(all);
                second.collect(all);
            }
        }
    }
    fn check(&self, depth: usize) -> Result<(), String> {
        if depth > 16 {
            return Err("dock nesting exceeds 16".into());
        }
        match self {
            Self::Tabs { tabs, active } => {
                if tabs.is_empty() || tabs.len() > 32 || !tabs.contains(active) {
                    return Err("invalid dock tab group/active tab".into());
                }
            }
            Self::Split {
                ratio,
                first,
                second,
                ..
            } => {
                if !ratio.is_finite() || !(0.10..=0.90).contains(ratio) {
                    return Err("invalid dock splitter ratio".into());
                }
                first.check(depth + 1)?;
                second.check(depth + 1)?;
            }
        }
        Ok(())
    }
    fn without(self, panel: DockPanel) -> Option<Self> {
        match self {
            Self::Tabs { mut tabs, active } => {
                tabs.retain(|p| *p != panel);
                let next_active = if tabs.contains(&active) {
                    active
                } else {
                    *tabs.first()?
                };
                Some(Self::Tabs {
                    tabs,
                    active: next_active,
                })
            }
            Self::Split {
                axis,
                ratio,
                first,
                second,
            } => {
                let first = first.without(panel);
                let second = second.without(panel);
                match (first, second) {
                    (Some(a), Some(b)) => Some(Self::Split {
                        axis,
                        ratio,
                        first: Box::new(a),
                        second: Box::new(b),
                    }),
                    (Some(a), None) | (None, Some(a)) => Some(a),
                    (None, None) => None,
                }
            }
        }
    }
    fn insert_tab(&mut self, panel: DockPanel, target: DockPanel) -> bool {
        match self {
            Self::Tabs { tabs, active } => {
                if tabs.contains(&target) {
                    tabs.push(panel);
                    *active = panel;
                    true
                } else {
                    false
                }
            }
            Self::Split { first, second, .. } => {
                first.insert_tab(panel, target) || second.insert_tab(panel, target)
            }
        }
    }
    fn split_around(
        &mut self,
        panel: DockPanel,
        target: DockPanel,
        axis: SplitAxis,
        before: bool,
    ) -> bool {
        match self {
            Self::Tabs { tabs, .. } if tabs.contains(&target) => {
                let previous = self.clone();
                let newcomer = Self::single(panel);
                let (first, second) = if before {
                    (newcomer, previous)
                } else {
                    (previous, newcomer)
                };
                *self = Self::Split {
                    axis,
                    ratio: 0.5,
                    first: Box::new(first),
                    second: Box::new(second),
                };
                true
            }
            Self::Split { first, second, .. } => {
                first.split_around(panel, target, axis, before)
                    || second.split_around(panel, target, axis, before)
            }
            _ => false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DockRegion {
    pub open: bool,
    pub extent: f32,
    pub root: Option<DockNode>,
}
impl DockRegion {
    fn new(extent: f32, root: DockNode, open: bool) -> Self {
        Self {
            open,
            extent,
            root: Some(root),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FloatingDock {
    pub panel: DockPanel,
    pub position: [f32; 2],
    pub size: [f32; 2],
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AnimationPresentation {
    Hidden,
    Compact,
    Expanded,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceLayout {
    pub schema: String,
    pub left: DockRegion,
    pub right: DockRegion,
    pub bottom: DockRegion,
    pub floating: Vec<FloatingDock>,
    pub layers_expanded: bool,
    pub animation: AnimationPresentation,
    pub preview_visible: bool,
    pub preview_zoom: f32,
    pub preview_pan: [f32; 2],
}
impl Default for WorkspaceLayout {
    fn default() -> Self {
        let left = DockNode::Split {
            axis: SplitAxis::Vertical,
            ratio: 0.34,
            first: Box::new(DockNode::single(DockPanel::PixelStudio)),
            second: Box::new(DockNode::Split {
                axis: SplitAxis::Vertical,
                ratio: 0.5,
                first: Box::new(DockNode::single(DockPanel::GameTools)),
                second: Box::new(DockNode::single(DockPanel::SceneNavigator)),
            }),
        };
        let right = DockNode::Split {
            axis: SplitAxis::Vertical,
            ratio: 0.5,
            first: Box::new(DockNode::single(DockPanel::Inspector)),
            second: Box::new(DockNode::single(DockPanel::AssetBrowser)),
        };
        let bottom = DockNode::Tabs {
            tabs: vec![
                DockPanel::Console,
                DockPanel::Problems,
                DockPanel::Jobs,
                DockPanel::Debug,
            ],
            active: DockPanel::Console,
        };
        Self {
            schema: WORKSPACE_SCHEMA.into(),
            left: DockRegion::new(250.0, left, true),
            right: DockRegion::new(300.0, right, true),
            bottom: DockRegion::new(180.0, bottom, false),
            floating: vec![],
            layers_expanded: false,
            animation: AnimationPresentation::Hidden,
            preview_visible: false,
            preview_zoom: 0.35,
            preview_pan: [0.0, 0.0],
        }
    }
}
impl WorkspaceLayout {
    pub fn validate(&self) -> Result<(), String> {
        if self.schema != WORKSPACE_SCHEMA {
            return Err("unrecognized workspace schema".into());
        }
        for (name, region, min, max) in [
            ("left", &self.left, 120.0, 900.0),
            ("right", &self.right, 120.0, 900.0),
            ("bottom", &self.bottom, 30.0, 700.0),
        ] {
            if !region.extent.is_finite() || !(min..=max).contains(&region.extent) {
                return Err(format!("{name} panel extent outside limits"));
            }
            if let Some(root) = &region.root {
                root.check(0)?;
            }
        }
        if !self.preview_zoom.is_finite()
            || !(0.05..=16.0).contains(&self.preview_zoom)
            || self.preview_pan.iter().any(|v| !v.is_finite())
        {
            return Err("invalid preview camera".into());
        }
        if self.floating.len() > 32 {
            return Err("too many floating docks".into());
        }
        let mut panels = Vec::new();
        for region in [&self.left, &self.right, &self.bottom] {
            if let Some(root) = &region.root {
                root.collect(&mut panels);
            }
        }
        for floating in &self.floating {
            panels.push(floating.panel);
            if floating
                .position
                .iter()
                .any(|v| !v.is_finite() || v.abs() > 100_000.0)
                || floating
                    .size
                    .iter()
                    .any(|v| !v.is_finite() || !(80.0..=4096.0).contains(v))
            {
                return Err("invalid floating dock geometry".into());
            }
        }
        let mut seen = BTreeSet::new();
        if panels.iter().any(|panel| !seen.insert(*panel)) {
            return Err("same panel appears in more than one dock".into());
        }
        // A panel may be temporarily absent when hidden. The permanent center canvas is never docked.
        Ok(())
    }

    /// One invalid persisted workspace cannot break Studio startup or authoring files.
    pub fn restored_or_default(mut self) -> Self {
        if self.validate().is_err() {
            return Self::default();
        }
        // UI state should never become an unbounded screen coordinate even after monitor changes.
        for floating in &mut self.floating {
            floating.position[0] = floating.position[0].clamp(-4096.0, 4096.0);
            floating.position[1] = floating.position[1].clamp(-4096.0, 4096.0);
        }
        self
    }

    pub fn contains(&self, panel: DockPanel) -> bool {
        [&self.left, &self.right, &self.bottom]
            .iter()
            .any(|r| r.root.as_ref().is_some_and(|n| n.contains(panel)))
            || self.floating.iter().any(|f| f.panel == panel)
    }

    fn remove_panel(&mut self, panel: DockPanel) {
        for region in [&mut self.left, &mut self.right, &mut self.bottom] {
            region.root = region.root.take().and_then(|node| node.without(panel));
        }
        self.floating.retain(|f| f.panel != panel);
    }

    /// Atomic tab move; on failure the original graph is unchanged.
    pub fn tab_with(&mut self, panel: DockPanel, target: DockPanel) -> Result<(), String> {
        if panel == target {
            return Err("cannot dock a tab onto itself".into());
        }
        let mut next = self.clone();
        next.remove_panel(panel);
        let mut inserted = false;
        for region in [&mut next.left, &mut next.right, &mut next.bottom] {
            if let Some(root) = &mut region.root {
                inserted |= root.insert_tab(panel, target);
            }
            if inserted {
                break;
            }
        }
        if !inserted {
            return Err("dock target is not present".into());
        }
        next.validate()?;
        *self = next;
        Ok(())
    }

    /// Atomic nested split move; axis defines the new split, before its first child.
    pub fn split_next_to(
        &mut self,
        panel: DockPanel,
        target: DockPanel,
        axis: SplitAxis,
        before: bool,
    ) -> Result<(), String> {
        if panel == target {
            return Err("cannot split a panel against itself".into());
        }
        let mut next = self.clone();
        next.remove_panel(panel);
        let mut inserted = false;
        for region in [&mut next.left, &mut next.right, &mut next.bottom] {
            if let Some(root) = &mut region.root {
                inserted |= root.split_around(panel, target, axis, before);
            }
            if inserted {
                break;
            }
        }
        if !inserted {
            return Err("split target is not present".into());
        }
        next.validate()?;
        *self = next;
        Ok(())
    }

    pub fn float(
        &mut self,
        panel: DockPanel,
        position: [f32; 2],
        size: [f32; 2],
    ) -> Result<(), String> {
        let mut next = self.clone();
        next.remove_panel(panel);
        next.floating.push(FloatingDock {
            panel,
            position,
            size,
        });
        next.validate()?;
        *self = next;
        Ok(())
    }

    /// Replacing editor profile is transient and must NOT be saved over the edit layout.
    pub fn pie_profile(&self) -> Self {
        let mut pie = self.clone();
        pie.left.open = false;
        pie.right.open = false;
        pie.bottom.open = true;
        pie.layers_expanded = false;
        pie.animation = AnimationPresentation::Hidden;
        pie.preview_visible = false;
        pie
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_are_canvas_first_and_have_unique_panels() {
        let layout = WorkspaceLayout::default();
        layout.validate().unwrap();
        assert!(layout
            .left
            .root
            .as_ref()
            .unwrap()
            .contains(DockPanel::SceneNavigator));
        assert!(layout
            .right
            .root
            .as_ref()
            .unwrap()
            .contains(DockPanel::AssetBrowser));
        assert!(!layout.bottom.open);
        assert!(!layout.layers_expanded);
    }
    #[test]
    fn nesting_and_tab_moves_are_atomic_and_persisted() {
        let mut layout = WorkspaceLayout::default();
        layout
            .tab_with(DockPanel::SceneNavigator, DockPanel::PixelStudio)
            .unwrap();
        layout
            .split_next_to(
                DockPanel::AssetBrowser,
                DockPanel::PixelStudio,
                SplitAxis::Horizontal,
                true,
            )
            .unwrap();
        let serialized = serde_json::to_string_pretty(&layout).unwrap();
        let read_back: WorkspaceLayout = serde_json::from_str(&serialized).unwrap();
        read_back.validate().unwrap();
        assert_eq!(layout, read_back);
        let before = layout.clone();
        assert!(layout
            .tab_with(DockPanel::PixelStudio, DockPanel::AnimationTimeline)
            .is_err());
        assert_eq!(layout, before);
    }
    #[test]
    fn floating_does_not_duplicate_a_panel() {
        let mut layout = WorkspaceLayout::default();
        layout
            .float(DockPanel::AssetBrowser, [140.0, 90.0], [420.0, 600.0])
            .unwrap();
        assert_eq!(layout.floating.len(), 1);
        assert!(!layout
            .right
            .root
            .as_ref()
            .unwrap()
            .contains(DockPanel::AssetBrowser));
        layout
            .tab_with(DockPanel::AssetBrowser, DockPanel::Inspector)
            .unwrap();
        assert!(layout.floating.is_empty());
        layout.validate().unwrap();
    }
    #[test]
    fn bad_layouts_do_not_destroy_authoring_data() {
        let mut corrupt = WorkspaceLayout::default();
        corrupt.right = corrupt.left.clone();
        assert!(corrupt.validate().is_err());
        assert_eq!(corrupt.restored_or_default(), WorkspaceLayout::default());
        let mut invalid_ratio = WorkspaceLayout::default();
        if let Some(DockNode::Split { ratio, .. }) = &mut invalid_ratio.left.root {
            *ratio = f32::NAN;
        }
        assert!(invalid_ratio.validate().is_err());
    }
    #[test]
    fn pie_restores_editor_layout_without_persistent_mutation() {
        let mut edit = WorkspaceLayout::default();
        edit.layers_expanded = true;
        edit.preview_visible = true;
        edit.animation = AnimationPresentation::Expanded;
        let before = edit.clone();
        let pie = edit.pie_profile();
        assert!(!pie.left.open && !pie.right.open && pie.bottom.open);
        assert!(!pie.preview_visible && !pie.layers_expanded);
        assert_eq!(edit, before);
    }
}
