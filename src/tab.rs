use egui_dock::TabViewer;

use crate::{AllNodeTemplates, MaterialEditorGraphState, MaterialEditorState};


pub struct Tab {
    pub graph_state: MaterialEditorState,
    pub user_state: MaterialEditorGraphState,
    pub material_name: String,
}
pub struct MaterialEditorTabViewer;
impl TabViewer for MaterialEditorTabViewer {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.material_name.as_str().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let _ = tab.graph_state.draw_graph_editor(
            ui,
            AllNodeTemplates,
            &mut tab.user_state,
            Vec::default(),
        );
    }
}
