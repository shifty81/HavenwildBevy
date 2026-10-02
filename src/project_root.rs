//! Runtime project authority. A packaged Studio must never keep using the old
//! compile-time workstation path after its executable/project is relocated.
use std::{
    env,
    path::{Path, PathBuf},
};

pub fn valid(path: &Path) -> bool {
    path.join("project/forgepy.project.json").is_file()
        && path
            .join("content/scenes/elizawy_mapping_certification.scene.json")
            .is_file()
        && path
            .join("content/catalog/core_source_manifest.json")
            .is_file()
}

fn root_in_ancestors(path: &Path) -> Option<PathBuf> {
    path.ancestors()
        .find(|candidate| valid(candidate))
        .map(Path::to_path_buf)
}

pub fn resolve() -> PathBuf {
    if let Some(explicit) = env::var_os("HAVENWILD_BEVY_ROOT") {
        let configured = PathBuf::from(explicit);
        if valid(&configured) {
            return configured;
        }
        panic!(
            "HAVENWILD_BEVY_ROOT is not a Havenwild Bevy source root: {}",
            configured.display()
        );
    }
    if let Ok(current) = env::current_dir() {
        if let Some(found) = root_in_ancestors(&current) {
            return found;
        }
    }
    if let Ok(exe) = env::current_exe() {
        if let Some(found) = root_in_ancestors(&exe) {
            return found;
        }
    }
    let compiled = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    if valid(&compiled) {
        return compiled;
    }
    panic!("Cannot locate Havenwild Bevy project root. Start Studio through PCC.cmd from the game folder, or configure HAVENWILD_BEVY_ROOT.");
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn existing_project_root_is_valid_and_derived_from_nested_path() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR"));
        assert!(valid(root));
        assert_eq!(
            root_in_ancestors(&root.join("content/scenes")),
            Some(root.to_path_buf())
        );
    }
    #[test]
    fn unrelated_folder_is_not_a_project() {
        assert!(!valid(
            &env::temp_dir().join("havenwild_non_project_fixture")
        ));
    }
}
