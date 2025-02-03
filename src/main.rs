#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // its an example

use std::{borrow::Cow, num::NonZero, thread::current};

use eframe::egui;
use egui::{
    include_image, CentralPanel, DragValue, Image, Rect, Sense, SidePanel, Slider, Stroke, TopBottomPanel
};
use egui_dock::{DockArea, DockState, TabViewer};
use egui_node_graph2::{
    DataTypeTrait, GraphEditorState, Node, NodeDataTrait, NodeTemplateIter, NodeTemplateTrait,
    PanZoom, UserResponseTrait, WidgetValueTrait,
};
use serde::{Deserialize, Serialize};
use strum::VariantArray;
use strum_macros::VariantArray;
use tab::Tab;

mod ui;
mod tab;
fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Material Editor",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(MaterialEditorApp::new()))
        }),
    )
}

struct MaterialEditorApp {
    dock_state: DockState<Tab>,
}

impl MaterialEditorApp {
    fn new() -> Self {
        let mut materials = vec![];
        for material in std::fs::read_dir("./materials").unwrap() {
            materials.push(Tab {
                graph_state: Self::load(material.as_ref().unwrap().path().to_str().unwrap())
                    .unwrap_or_default(),
                user_state: MaterialEditorGraphState::default(),
                material_name: material
                    .as_ref()
                    .unwrap()
                    .file_name()
                    .into_string()
                    .unwrap(),
            });
        }
        Self {
            dock_state: DockState::new(materials),
        }
    }
    fn load(path: &str) -> Option<MaterialEditorState> {
        match std::fs::read(path) {
            Ok(editor_state_bytes) => {
                let ed: MaterialEditorState =
                    serde_json::from_str(&String::from_utf8(editor_state_bytes).unwrap())
                        .unwrap_or_default();
                Some(ed)
            }
            _ => None,
        }
    }

    fn save(&mut self) -> Result<(), std::io::Error> {
        for (_, tab) in self.dock_state.iter_all_tabs() {
            std::fs::write(
                format!("./materials/{}", tab.material_name),
                serde_json::to_string(&tab.graph_state).unwrap(),
            )
            .unwrap();
        }
        Ok(())
    }
    
    fn get_current_tab_mut(&mut self) -> &mut Tab {
        &mut self
            .dock_state
            .main_surface_mut()
            .root_node_mut()
            .unwrap()
            .tabs_mut()
            .unwrap()[0]
    }
}
#[derive(Debug, Clone, Copy, VariantArray, Deserialize, Serialize)]

pub enum MaterialEditorNodeTemplate {
    Constant,
    Add,
    TexCoord,
    TextureSample,
    MakeFVec2,
}
#[derive(Debug, Default)]
pub struct MaterialEditorGraphState {}
type MaterialEditorState = GraphEditorState<
    MaterialEditorNodeData,
    MaterialEditorDataType,
    MaterialEditorValueType,
    MaterialEditorNodeTemplate,
    MaterialEditorGraphState,
>;
#[derive(Debug, Deserialize, Serialize)]
pub struct MaterialEditorNodeData {
    template: MaterialEditorNodeTemplate,
}
#[derive(Debug, PartialEq, Eq, Deserialize, Serialize)]
pub enum MaterialEditorDataType {
    Scalar,
    Vec2,
    Vec3,
}
#[derive(Debug, Deserialize, Serialize)]
pub enum MaterialEditorValueType {
    F32 { value: f32 },
    FVec2(f32, f32),
    TextureId(u32),
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MaterialEditorResponse {}
impl Default for MaterialEditorValueType {
    fn default() -> Self {
        Self::F32 { value: 0.0 }
    }
}
impl DataTypeTrait<MaterialEditorGraphState> for MaterialEditorDataType {
    fn data_type_color(&self, user_state: &mut MaterialEditorGraphState) -> egui::Color32 {
        egui::Color32::from_rgb(222, 222, 222)
    }

    fn name(&self) -> std::borrow::Cow<str> {
        match self {
            MaterialEditorDataType::Scalar => Cow::Borrowed("scalar"),
            MaterialEditorDataType::Vec2 => Cow::Borrowed("Vec2"),
            MaterialEditorDataType::Vec3 => Cow::Borrowed("Vec3"),
        }
    }
}

impl NodeTemplateTrait for MaterialEditorNodeTemplate {
    type NodeData = MaterialEditorNodeData;

    type DataType = MaterialEditorDataType;

    type ValueType = MaterialEditorValueType;

    type UserState = MaterialEditorGraphState;

    type CategoryType = &'static str;

    fn node_finder_label(&self, user_state: &mut Self::UserState) -> std::borrow::Cow<str> {
        Cow::Borrowed(match self {
            MaterialEditorNodeTemplate::Constant => "constant",
            MaterialEditorNodeTemplate::Add => "add",
            MaterialEditorNodeTemplate::TexCoord => "texcoord",
            MaterialEditorNodeTemplate::TextureSample => "texture sample",
            MaterialEditorNodeTemplate::MakeFVec2 => "make fvec2",
        })
    }
    fn node_finder_categories(&self, _user_state: &mut Self::UserState) -> Vec<Self::CategoryType> {
        match self {
            MaterialEditorNodeTemplate::TexCoord | MaterialEditorNodeTemplate::TextureSample => {
                vec!["Texture"]
            }
            _ => vec!["Math"],
        }
    }

    fn node_graph_label(&self, user_state: &mut Self::UserState) -> String {
        self.node_finder_label(user_state).to_string()
    }

    fn user_data(&self, user_state: &mut Self::UserState) -> Self::NodeData {
        MaterialEditorNodeData { template: *self }
    }

    fn build_node(
        &self,
        graph: &mut egui_node_graph2::Graph<Self::NodeData, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
        node_id: egui_node_graph2::NodeId,
    ) {
        match self {
            MaterialEditorNodeTemplate::Constant => {
                graph.add_input_param(
                    node_id,
                    "constant".to_string(),
                    MaterialEditorDataType::Scalar,
                    MaterialEditorValueType::F32 { value: 50.0 },
                    egui_node_graph2::InputParamKind::ConstantOnly,
                    true,
                );
                graph.add_output_param(
                    node_id,
                    "output".to_string(),
                    MaterialEditorDataType::Scalar,
                );
            }
            MaterialEditorNodeTemplate::Add => {
                graph.add_input_param(
                    node_id,
                    "a".to_string(),
                    MaterialEditorDataType::Scalar,
                    MaterialEditorValueType::F32 { value: 0.0 },
                    egui_node_graph2::InputParamKind::ConnectionOrConstant,
                    true,
                );
                graph.add_input_param(
                    node_id,
                    "b".to_string(),
                    MaterialEditorDataType::Scalar,
                    MaterialEditorValueType::F32 { value: 0.0 },
                    egui_node_graph2::InputParamKind::ConnectionOrConstant,
                    true,
                );
                graph.add_output_param(node_id, "out".to_string(), MaterialEditorDataType::Scalar);
            }
            MaterialEditorNodeTemplate::TexCoord => {
                graph.add_output_param(
                    node_id,
                    "uv output".to_string(),
                    MaterialEditorDataType::Vec2,
                );
            }
            MaterialEditorNodeTemplate::TextureSample => {
                graph.add_input_param(
                    node_id,
                    "texture id".to_string(),
                    MaterialEditorDataType::Scalar,
                    MaterialEditorValueType::TextureId(0),
                    egui_node_graph2::InputParamKind::ConnectionOrConstant,
                    true,
                );

                graph.add_input_param(
                    node_id,
                    "coord".to_string(),
                    MaterialEditorDataType::Vec2,
                    MaterialEditorValueType::FVec2(0.0, 0.0),
                    egui_node_graph2::InputParamKind::ConnectionOrConstant,
                    true,
                );
                graph.add_output_param(
                    node_id,
                    "rgb output".to_string(),
                    MaterialEditorDataType::Vec3,
                );
            }
            MaterialEditorNodeTemplate::MakeFVec2 => {
                graph.add_input_param(
                    node_id,
                    "x".to_string(),
                    MaterialEditorDataType::Scalar,
                    MaterialEditorValueType::F32 { value: 0.0 },
                    egui_node_graph2::InputParamKind::ConnectionOrConstant,
                    true,
                );
                graph.add_input_param(
                    node_id,
                    "y".to_string(),
                    MaterialEditorDataType::Scalar,
                    MaterialEditorValueType::F32 { value: 0.0 },
                    egui_node_graph2::InputParamKind::ConnectionOrConstant,
                    true,
                );
                graph.add_output_param(
                    node_id,
                    "xy output".to_string(),
                    MaterialEditorDataType::Vec2,
                );
            }
        }
    }
}
pub struct AllNodeTemplates;
impl NodeTemplateIter for AllNodeTemplates {
    type Item = MaterialEditorNodeTemplate;

    fn all_kinds(&self) -> Vec<Self::Item> {
        MaterialEditorNodeTemplate::VARIANTS.to_vec()
    }
}
impl WidgetValueTrait for MaterialEditorValueType {
    type Response = MaterialEditorResponse;

    type UserState = MaterialEditorGraphState;

    type NodeData = MaterialEditorNodeData;

    fn value_widget(
        &mut self,
        param_name: &str,
        node_id: egui_node_graph2::NodeId,
        ui: &mut egui::Ui,
        user_state: &mut Self::UserState,
        node_data: &Self::NodeData,
    ) -> Vec<Self::Response> {
        match self {
            MaterialEditorValueType::F32 { value } => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.add(DragValue::new(value));
                });
            }
            MaterialEditorValueType::TextureId(id) => {
                ui.label(format!("{} {}", param_name, id));
            }
            MaterialEditorValueType::FVec2(x, y) => {
                ui.label(format!("{} {} {}", param_name, x, y));
            }
        }
        vec![]
    }
}
impl UserResponseTrait for MaterialEditorResponse {}
impl NodeDataTrait for MaterialEditorNodeData {
    type Response = MaterialEditorResponse;

    type UserState = MaterialEditorGraphState;

    type DataType = MaterialEditorDataType;

    type ValueType = MaterialEditorValueType;

    fn bottom_ui(
        &self,
        ui: &mut egui::Ui,
        node_id: egui_node_graph2::NodeId,
        graph: &egui_node_graph2::Graph<Self, Self::DataType, Self::ValueType>,
        user_state: &mut Self::UserState,
    ) -> Vec<egui_node_graph2::NodeResponse<Self::Response, Self>>
    where
        Self::Response: egui_node_graph2::UserResponseTrait,
    {
        vec![]
    }
}
