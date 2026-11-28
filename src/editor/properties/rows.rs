use crate::widgets::tree::TreeRow;
use std::rc::Rc;

fn num<T: egui::emath::Numeric>(ui: &mut egui::Ui, n: &mut T, prefix: &str) -> egui::Response {
    ui.add(
        egui::DragValue::new(n)
            .speed(0.1)
            .max_decimals(3)
            .prefix(prefix),
    )
}

pub fn rows(ui: &mut egui::Ui, name_width: f32, label: &str, value: &mut ree::Value) -> bool {
    let row_height = TreeRow::height(ui);

    let row = |depth| TreeRow {
        depth,
        has_children: false,
        expanded: false,
        selected: false,
    };

    let name_cell = |ui: &mut egui::Ui, label: &str| {
        ui.allocate_ui_with_layout(
            egui::vec2(name_width, row_height),
            egui::Layout::left_to_right(egui::Align::Center),
            |ui| {
                ui.set_min_width(name_width);
                ui.add(egui::Label::new(label).truncate());
            },
        );
    };

    let mut changed = false;

    match value {
        ree::Value::Bool(v) => {
            row(0).show(ui, |ui| {
                name_cell(ui, label);
                changed = ui.checkbox(v, ()).changed();
            });
        }

        ree::Value::Number(v) => {
            row(0).show(ui, |ui| {
                name_cell(ui, label);
                changed = num(ui, v, "").changed();
            });
        }

        ree::Value::String(v) => {
            row(0).show(ui, |ui| {
                name_cell(ui, label);
                let mut text = v.to_str_lossy().to_string();
                changed = ui.text_edit_singleline(&mut text).changed();
                if changed {
                    *v = text.as_str().into();
                }
            });
        }

        ree::Value::Color3(v) => {
            row(0).show(ui, |ui| {
                name_cell(ui, label);
                let mut rgb = [v.r, v.g, v.b];
                changed = ui.color_edit_button_rgb(&mut rgb).changed();
                if changed {
                    *v = rbx_dom_weak::types::Color3::new(rgb[0], rgb[1], rgb[2]);
                }
            });
        }

        ree::Value::Vector2(v) => {
            row(0).show(ui, |ui| {
                name_cell(ui, label);
                changed = (num(ui, &mut v.x, "X ") | num(ui, &mut v.y, "Y ")).changed();
            });
        }

        ree::Value::Vector3(v) => {
            row(0).show(ui, |ui| {
                name_cell(ui, label);
                changed =
                    (num(ui, &mut v.x, "X ") | num(ui, &mut v.y, "Y ") | num(ui, &mut v.z, "Z "))
                        .changed();
            });
        }

        ree::Value::Rect(v) => {
            row(0).show(ui, |ui| name_cell(ui, label));
            for (part, corner) in [("Min", &mut v.min), ("Max", &mut v.max)] {
                row(1).show(ui, |ui| {
                    name_cell(ui, part);
                    changed |=
                        (num(ui, &mut corner.x, "X ") | num(ui, &mut corner.y, "Y ")).changed();
                });
            }
        }

        ree::Value::Dom(v) => match Rc::make_mut(v) {
            rbx_dom_weak::types::Variant::CFrame(cf) => {
                row(0).show(ui, |ui| {
                    name_cell(ui, label);
                    let p = cf.position;
                    ui.weak(format!("{}, {}, {}", p.x, p.y, p.z));
                });
                row(1).show(ui, |ui| {
                    name_cell(ui, "Position");
                    let p = &mut cf.position;
                    changed |= (num(ui, &mut p.x, "X ")
                        | num(ui, &mut p.y, "Y ")
                        | num(ui, &mut p.z, "Z "))
                    .changed();
                });
            }

            rbx_dom_weak::types::Variant::UDim2(v) => {
                row(0).show(ui, |ui| name_cell(ui, label));
                for (part, udim) in [("X", &mut v.x), ("Y", &mut v.y)] {
                    row(1).show(ui, |ui| {
                        name_cell(ui, part);
                        changed |= (num(ui, &mut udim.scale, "Scale ")
                            | num(ui, &mut udim.offset, "Offset "))
                        .changed();
                    });
                }
            }

            rbx_dom_weak::types::Variant::UDim(v) => {
                row(0).show(ui, |ui| {
                    name_cell(ui, label);
                    changed = (num(ui, &mut v.scale, "Scale ") | num(ui, &mut v.offset, "Offset "))
                        .changed();
                });
            }

            unknown => {
                row(0).show(ui, |ui| {
                    name_cell(ui, label);
                    ui.add(
                        egui::Label::new(egui::RichText::new(format!("{unknown:?}")).weak())
                            .truncate(),
                    );
                });
            }
        },

        unknown => {
            row(0).show(ui, |ui| {
                name_cell(ui, label);
                ui.add(
                    egui::Label::new(egui::RichText::new(format!("{unknown:?}")).weak()).truncate(),
                );
            });
        }
    }

    changed
}
