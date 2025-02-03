use egui::{include_image, CentralPanel, SidePanel, TopBottomPanel};
use egui_dock::DockArea;
use egui_node_graph2::NodeTemplateTrait;

use crate::{tab::MaterialEditorTabViewer, MaterialEditorApp, MaterialEditorNodeData, MaterialEditorNodeTemplate};

impl eframe::App for MaterialEditorApp {
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
        // TopBottomPanel::bottom("content_browser")
        //     .min_height(bottom_height)
        //     .show(ctx, |ui| {
        //         ui.heading("Content Browser");
        //         ui.label("Assets go here...");
        //     });

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
                    for i in 0..5 {
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
                    .show_inside(ui, &mut MaterialEditorTabViewer);
            })
        });
    }
}