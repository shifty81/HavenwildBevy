//! Local editor layout persistence. Layout state is workstation-local and lives under .forgepy/.

use crate::atomic_file::write_atomic;
use forge_gui_chrome::{ModularSurfaceState, SurfaceDock};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EditorLayout {
    pub schema: String,
    pub terrain_mapper: ModularSurfaceState,
    #[serde(default = "default_mapper_tab")]
    pub terrain_mapper_tab: String,
    #[serde(default)]
    pub layers_open: bool,
    #[serde(default)]
    pub inspect_alpha: bool,
    #[serde(default = "default_rulers_visible")]
    pub rulers_visible: bool,
    #[serde(default = "default_mapping_status_visible")]
    pub mapping_status_visible: bool,
}

fn default_mapper_tab() -> String {
    "recipe".into()
}

fn default_rulers_visible() -> bool {
    true
}
fn default_mapping_status_visible() -> bool {
    true
}

impl Default for EditorLayout {
    fn default() -> Self {
        let mut terrain_mapper = ModularSurfaceState::new(
            "havenwild.terrain_mapper",
            "Terrain Mapper",
            SurfaceDock::Floating,
        );
        terrain_mapper.preferred_size = [440.0, 720.0];
        terrain_mapper.visible = false;
        Self {
            schema: "havenwild.bevy.editor_layout.local.v1".into(),
            terrain_mapper,
            terrain_mapper_tab: default_mapper_tab(),
            layers_open: false,
            inspect_alpha: false,
            rulers_visible: true,
            mapping_status_visible: true,
        }
    }
}

impl EditorLayout {
    pub fn load(path: &Path) -> Self {
        fs::read_to_string(path)
            .ok()
            .and_then(|text| serde_json::from_str::<Self>(&text).ok())
            .filter(|layout| layout.schema == "havenwild.bevy.editor_layout.local.v1")
            .unwrap_or_default()
    }

    pub fn save(&self, path: &Path) -> Result<(), String> {
        let text = serde_json::to_string_pretty(self).map_err(|e| e.to_string())? + "\n";
        write_atomic(path, text.as_bytes())
    }
}

#[cfg(test)]
mod scene_first_layout_tests {
    use super::*;

    #[test]
    fn fresh_studio_starts_unobstructed() {
        let layout = EditorLayout::default();
        assert!(!layout.terrain_mapper.visible);
        assert_eq!(layout.terrain_mapper.dock, SurfaceDock::Floating);
        assert!(!layout.layers_open);
        assert!(!layout.inspect_alpha);
        assert!(layout.rulers_visible);
        assert!(layout.mapping_status_visible);
    }

    #[test]
    fn previous_layout_json_loads_new_optional_fields_as_false() {
        let mut json = serde_json::to_value(EditorLayout::default()).unwrap();
        let object = json.as_object_mut().unwrap();
        object.remove("layersOpen");
        object.remove("inspectAlpha");
        object.remove("rulersVisible");
        object.remove("mappingStatusVisible");
        let legacy: EditorLayout = serde_json::from_value(json).unwrap();
        assert!(!legacy.layers_open);
        assert!(!legacy.inspect_alpha);
        assert!(legacy.rulers_visible);
        assert!(legacy.mapping_status_visible);
    }
}
