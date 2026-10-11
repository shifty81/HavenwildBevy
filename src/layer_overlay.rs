//! Session-only layer isolate control; the seven authored visual layers remain
//! owned by EditorLayout and SceneV2. This helper only preserves the temporary
//! pre-isolate visibility so Shift-click is reversible instead of destructive.

#[derive(Default, Debug)]
pub struct LayerIsolateState {
    saved_visibility: Option<[bool; 7]>,
    isolated_slot: Option<usize>,
}

impl LayerIsolateState {
    /// Shift-click an icon to isolate; repeat on that icon to restore exactly
    /// the visibility flags that were present before the first isolation.
    pub fn toggle(&mut self, visibility: &mut [bool; 7], slot: usize) -> bool {
        if slot >= visibility.len() {
            return false;
        }
        if self.isolated_slot == Some(slot) {
            if let Some(saved) = self.saved_visibility.take() {
                *visibility = saved;
            }
            self.isolated_slot = None;
            return false;
        }
        if self.saved_visibility.is_none() {
            self.saved_visibility = Some(*visibility);
        }
        *visibility = [false; 7];
        visibility[slot] = true;
        self.isolated_slot = Some(slot);
        true
    }

    /// The isolation overlay is temporary. Any layout save (theme, panel size,
    /// open/close, etc.) must persist the pre-isolate view, not this temporary
    /// presentation. On restart the original visibility is restored by loading
    /// the saved layout, without persisting stale session state.
    pub fn persisted_visibility(&self, visible: [bool; 7]) -> [bool; 7] {
        self.saved_visibility.unwrap_or(visible)
    }

    /// A manual eye toggle breaks isolation; the user's new arrangement wins.
    pub fn cancel(&mut self) {
        self.saved_visibility = None;
        self.isolated_slot = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shift_isolate_restores_exact_hidden_layer_preferences() {
        let mut state = LayerIsolateState::default();
        let original = [true, false, true, true, false, true, true];
        let mut visible = original;
        assert!(state.toggle(&mut visible, 2));
        assert_eq!(visible, [false, false, true, false, false, false, false]);
        assert!(state.toggle(&mut visible, 4));
        assert_eq!(visible, [false, false, false, false, true, false, false]);
        assert!(!state.toggle(&mut visible, 4));
        assert_eq!(visible, original);
    }

    #[test]
    fn manual_eye_toggle_prevents_restoring_stale_snapshot() {
        let mut state = LayerIsolateState::default();
        let mut visible = [true; 7];
        assert!(state.toggle(&mut visible, 0));
        visible[1] = true;
        state.cancel();
        assert!(state.toggle(&mut visible, 1));
        assert!(!state.toggle(&mut visible, 1));
        assert_eq!(visible[1], true);
        assert_eq!(visible[0], true);
        assert_eq!(visible.iter().filter(|v| **v).count(), 2);
    }

    #[test]
    fn isolating_then_saving_an_unrelated_setting_preserves_disk_visibility() {
        let mut isolate = LayerIsolateState::default();
        let originally_visible = [true, false, true, false, true, false, true];
        let mut visible = originally_visible;
        assert!(isolate.toggle(&mut visible, 1));
        assert_eq!(visible, [false, true, false, false, false, false, false]);
        // Layout save uses the pre-isolate state. Restart reconstructs from the
        // persisted value; isolation never mutates the saved layer preference.
        let persisted = isolate.persisted_visibility(visible);
        assert_eq!(persisted, originally_visible);
        let restarted = LayerIsolateState::default();
        assert_eq!(
            restarted.persisted_visibility(persisted),
            originally_visible
        );
        assert!(!isolate.toggle(&mut visible, 1));
        assert_eq!(visible, persisted);
        assert_eq!(isolate.persisted_visibility(visible), persisted);
    }

    #[test]
    fn invalid_slot_cannot_change_visibility() {
        let mut state = LayerIsolateState::default();
        let mut visible = [true; 7];
        assert!(!state.toggle(&mut visible, 7));
        assert_eq!(visible, [true; 7]);
    }
}
