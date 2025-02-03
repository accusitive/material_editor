#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")] // hide console window on Windows in release
#![allow(rustdoc::missing_crate_level_docs)] // its an example

use std::{borrow::Cow, num::NonZero, thread::current};

use eframe::egui;
use egui::{
    include_image, CentralPanel, DragValue, Image, Rect, Sense, SidePanel, Stroke, TopBottomPanel,
};
use egui_dock::{DockArea, DockState, TabViewer};
use egui_node_graph2::{
    DataTypeTrait, GraphEditorState, Node, NodeDataTrait, NodeTemplateIter, NodeTemplateTrait,
    PanZoom, UserResponseTrait, WidgetValueTrait,
};
use serde::{Deserialize, Serialize};

fn main() -> eframe::Result {
    env_logger::init(); // Log to stderr (if you run with `RUST_LOG=debug`).
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([320.0, 240.0]),
        ..Default::default()
    };

    eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Ok(Box::new(MyApp::new()))
        }),
    )
}

struct MyApp {
    dock_state: DockState<Tab>,
}

impl MyApp {
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
    fn load(path: &str) -> Option<MyEditorState> {
        match std::fs::read(path) {
            Ok(editor_state_bytes) => {
                let ed: MyEditorState =
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
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]

pub enum MaterialEditorNodeTemplate {
    Constant,
    Add,
    TexCoord,
    TextureSample,
    MakeFVec2,
}
#[derive(Debug, Default)]
pub struct MaterialEditorGraphState {}
type MyEditorState = GraphEditorState<
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
    I32 { value: i32 },
    FVec2(f32, f32),
    TextureId(u32),
}
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum MaterialEditorResponse {}
impl Default for MaterialEditorValueType {
    fn default() -> Self {
        Self::I32 { value: 0 }
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
                    MaterialEditorValueType::I32 { value: 0 },
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
                    MaterialEditorValueType::I32 { value: 0 },
                    egui_node_graph2::InputParamKind::ConnectionOrConstant,
                    true,
                );
                graph.add_input_param(
                    node_id,
                    "b".to_string(),
                    MaterialEditorDataType::Scalar,
                    MaterialEditorValueType::I32 { value: 0 },
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
                    MaterialEditorValueType::I32 { value: 0 },
                    egui_node_graph2::InputParamKind::ConnectionOrConstant,
                    true,
                );
                graph.add_input_param(
                    node_id,
                    "y".to_string(),
                    MaterialEditorDataType::Scalar,
                    MaterialEditorValueType::I32 { value: 0 },
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
pub struct AllMyNodeTemplates;
impl NodeTemplateIter for AllMyNodeTemplates {
    type Item = MaterialEditorNodeTemplate;

    fn all_kinds(&self) -> Vec<Self::Item> {
        vec![
            MaterialEditorNodeTemplate::Constant,
            MaterialEditorNodeTemplate::Add,
            MaterialEditorNodeTemplate::TexCoord,
            MaterialEditorNodeTemplate::TextureSample,
            MaterialEditorNodeTemplate::MakeFVec2,
        ]
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
            MaterialEditorValueType::I32 { mut value } => {
                ui.horizontal(|ui| {
                    ui.label(param_name);
                    ui.add(DragValue::new(&mut value).speed(5.0));
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
impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let screen_rect = ctx.screen_rect();
        let width = screen_rect.width();
        let height = screen_rect.height();

        let left_width = width * 0.2; // 1/5 of total width
        let right_width = width * 0.2; // 1/5 of total width
        let center_width = width * 0.4; // 2/5 of total width
        let bottom_height = height * 0.4; // 2/5 of total height

        // Top menu bar
        TopBottomPanel::top("top_bar").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Save").clicked() {
                        self.save().unwrap();
                    }
                    // if ui.button("Open").clicked() {
                    // }
                });
                ui.menu_button("Editor", |ui| {
                    if ui.button("Reset Pan/Zoom").clicked() {
                        // for material_editor_state in self.material_editor_states {
                        //     material_editor_state.pan_zoom = PanZoom::default();
                        // }
                    }
                })
            });
        });

        // Bottom panel for Content Browser
        TopBottomPanel::bottom("content_browser")
            .min_height(bottom_height)
            .show(ctx, |ui| {
                ui.heading("Content Browser");
                ui.label("Assets go here...");
            });

        // Left panel for Material Properties
        SidePanel::left("material_properties")
            .exact_width(left_width)
            .show(ctx, |ui| {
                ui.heading("Material Properties");
                ui.label("Properties go here...");
            });

        // Right panel for Texture References
        SidePanel::right("texture_references")
            .exact_width(right_width)
            .show(ctx, |ui| {
                egui::ScrollArea::vertical().show(ui, |ui| {
                    for i in 0..50 {
                        let response = ui.dnd_drag_source(egui::Id::new(i), (), |ui| {
                            ui.label(format!("Texture Reference here {}", i));
                            let source = include_image!("../test.jpg");
                            let image = egui::Image::new(source);
                            ui.add(image.max_width(128.0).max_height(128.0));
                        });

                        let current_tab = self.get_current_tab_mut();

                        if response.response.drag_stopped() {
                            let pointer = ui.input(|i| i.pointer.interact_pos().unwrap());
                            dbg!(&pointer, &current_tab.graph_state.pan_zoom.clip_rect);
                            if current_tab.graph_state.pan_zoom.clip_rect.contains(pointer) {
                                let template = MaterialEditorNodeTemplate::TextureSample;

                                current_tab.graph_state.graph.add_node(
                                    template.node_graph_label(&mut current_tab.user_state),
                                    MaterialEditorNodeData {
                                        template: MaterialEditorNodeTemplate::TextureSample,
                                    },
                                    |graph, node_id| {
                                        current_tab.graph_state.node_order.push(node_id);

                                        let pan = current_tab.graph_state.pan_zoom.pan; // vec2 containing pan info
                                        let clip_rect = current_tab.graph_state.pan_zoom.clip_rect;

                                        let graph_pos = egui::Pos2 {
                                            x: ((pointer.x - clip_rect.x_range().min) - pan.x),
                                            y: ((pointer.y - clip_rect.y_range().min) - pan.y),
                                        };
                                        current_tab
                                            .graph_state
                                            .node_positions
                                            .insert(node_id, graph_pos);
                                        MaterialEditorNodeTemplate::TextureSample.build_node(
                                            graph,
                                            &mut current_tab.user_state,
                                            node_id,
                                        );
                                    },
                                );
                            }
                        }
                    }
                });
            });

        // Main content: Graph Editor
        CentralPanel::default().show(ctx, |ui| {
            egui::CentralPanel::default().show(ctx, |ui| {
                DockArea::new(&mut self.dock_state)
                    .draggable_tabs(false)
                    .show_inside(ui, &mut MyTabViewer);
            })
        });
    }
}
struct Tab {
    graph_state: MyEditorState,
    user_state: MaterialEditorGraphState,
    material_name: String,
}
// type Tab = (MyEditorState, MaterialEditorGraphState);
struct MyTabViewer;
impl TabViewer for MyTabViewer {
    type Tab = Tab;

    fn title(&mut self, tab: &mut Self::Tab) -> egui::WidgetText {
        tab.material_name.as_str().into()
    }

    fn ui(&mut self, ui: &mut egui::Ui, tab: &mut Self::Tab) {
        let _ = tab.graph_state.draw_graph_editor(
            ui,
            AllMyNodeTemplates,
            &mut tab.user_state,
            Vec::default(),
        );
    }
}
