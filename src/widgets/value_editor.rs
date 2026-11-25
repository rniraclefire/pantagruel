use glam::{EulerRot, Mat3, Vec3};
use rbx_dom_weak::types::{CFrame, Color3, Matrix3, UDim, Variant, Vector2, Vector3};
use ree::{Value, reflection::Reflection};

use super::row::row;

pub fn property(
    ui: &mut egui::Ui,
    id: egui::Id,
    reflection: &Reflection,
    name: &str,
    value: &Value,
) -> Option<Value> {
    match value {
        Value::Bool(v) => {
            let mut v = *v;
            inline(ui, name, |ui| ui.checkbox(&mut v, "").changed()).then(|| Value::Bool(v))
        }

        Value::Number(v) => {
            let mut v = *v;
            inline(ui, name, |ui| num(ui, &mut v, "").changed()).then(|| Value::Number(v))
        }

        Value::String(v) => {
            let mut typed = None;
            row(ui, name, |ui| typed = text(ui, id, &v.to_string()));
            typed.map(|t| Value::String(t.into()))
        }

        Value::Color3(v) => {
            let mut rgb = [v.r, v.g, v.b];
            inline(ui, name, |ui| ui.color_edit_button_rgb(&mut rgb).changed())
                .then(|| Value::Color3(Color3::new(rgb[0], rgb[1], rgb[2])))
        }

        Value::Rect(v) => {
            let mut v = *v;
            let summary = join(&[v.min.x, v.min.y, v.max.x, v.max.y]);
            let (typed, changed) = group(ui, id, name, summary, |ui| {
                inline(ui, "Min", |ui| vector2(ui, &mut v.min))
                    | inline(ui, "Max", |ui| vector2(ui, &mut v.max))
            });
            let parsed = typed.map(|t| numbers(&t));
            let typed = if let Some(&[a, b, c, d]) = parsed.as_deref() {
                v.min = Vector2::new(a, b);
                v.max = Vector2::new(c, d);
                true
            } else {
                false
            };
            (changed | typed).then(|| Value::Rect(v))
        }

        Value::Vector2(v) => {
            let mut v = *v;
            inline(ui, name, |ui| vector2(ui, &mut v)).then(|| Value::Vector2(v))
        }

        Value::Vector3(v) => {
            let mut v = *v;
            inline(ui, name, |ui| vector3(ui, &mut v)).then(|| Value::Vector3(v))
        }

        Value::EnumItem(v) => {
            let info = reflection.enum_item_info(*v);
            let mut current = *v;
            inline(ui, name, |ui| {
                egui::ComboBox::from_id_salt(id)
                    .selected_text(&*info.name)
                    .show_ui(ui, |ui| {
                        for &item in &reflection.enums[info.enum_type as usize].items {
                            ui.selectable_value(
                                &mut current,
                                item,
                                &*reflection.enum_item_info(item).name,
                            );
                        }
                    });
                current != *v
            })
            .then(|| Value::EnumItem(current))
        }

        Value::Dom(v) => match &**v {
            Variant::CFrame(cf) => {
                let mut position = cf.position;
                let mut orientation = to_orientation(cf.orientation);
                let mut rotated = false;
                let summary = join(&[position.x, position.y, position.z]);
                let (typed, moved) = group(ui, id, name, summary, |ui| {
                    let moved = inline(ui, "Position", |ui| vector3(ui, &mut position));
                    rotated = inline(ui, "Orientation", |ui| vector3(ui, &mut orientation));
                    moved | rotated
                });
                let parsed = typed.map(|t| numbers(&t));
                let typed = if let Some(&[x, y, z]) = parsed.as_deref() {
                    position = Vector3::new(x, y, z);
                    true
                } else {
                    false
                };
                (moved | typed).then(|| {
                    let rotation = if rotated {
                        from_orientation(orientation)
                    } else {
                        cf.orientation
                    };
                    Value::Dom(Variant::CFrame(CFrame::new(position, rotation)).into())
                })
            }

            Variant::UDim2(v) => {
                let mut v = *v;
                let summary = format!(
                    "{{{}}}, {{{}}}",
                    join(&[v.x.scale, v.x.offset as f32]),
                    join(&[v.y.scale, v.y.offset as f32]),
                );
                let (typed, changed) = group(ui, id, name, summary, |ui| {
                    udim(ui, "X", &mut v.x) | udim(ui, "Y", &mut v.y)
                });
                let parsed = typed.map(|t| numbers(&t));
                let typed = if let Some(&[xs, xo, ys, yo]) = parsed.as_deref() {
                    v.x = UDim::new(xs, xo.round() as i32);
                    v.y = UDim::new(ys, yo.round() as i32);
                    true
                } else {
                    false
                };
                (changed | typed).then(|| Value::Dom(Variant::UDim2(v).into()))
            }

            Variant::UDim(v) => {
                let mut v = *v;
                udim(ui, name, &mut v).then(|| Value::Dom(Variant::UDim(v).into()))
            }

            _ => fallback(ui, name, value),
        },

        _ => fallback(ui, name, value),
    }
}

fn inline(ui: &mut egui::Ui, name: &str, edit: impl FnOnce(&mut egui::Ui) -> bool) -> bool {
    let mut changed = false;
    row(ui, name, |ui| changed = edit(ui));
    changed
}

/// A row with the value as editable text, and its parts as rows below it.
/// Returns the text typed into the row (if committed) and whether a part changed.
fn group(
    ui: &mut egui::Ui,
    id: egui::Id,
    name: &str,
    summary: String,
    body: impl FnOnce(&mut egui::Ui) -> bool,
) -> (Option<String>, bool) {
    let mut typed = None;
    row(ui, name, |ui| typed = text(ui, id, &summary));
    let changed = ui.indent(id, body).inner;
    (typed, changed)
}

/// A text field that commits when it loses focus.
fn text(ui: &mut egui::Ui, id: egui::Id, current: &str) -> Option<String> {
    let mut buffer = ui
        .data_mut(|d| d.get_temp::<String>(id))
        .unwrap_or_else(|| current.to_owned());
    let response = ui.add(
        egui::TextEdit::singleline(&mut buffer)
            .id(id)
            .desired_width(f32::INFINITY),
    );
    if response.has_focus() {
        ui.data_mut(|d| d.insert_temp(id, buffer));
        None
    } else {
        ui.data_mut(|d| d.remove::<String>(id));
        (response.lost_focus() && buffer != current).then_some(buffer)
    }
}

fn fallback(ui: &mut egui::Ui, name: &str, value: &Value) -> Option<Value> {
    row(ui, name, |ui| {
        ui.add(egui::Label::new(egui::RichText::new(format!("{value:?}")).weak()).truncate());
    });
    None
}

/// `1, 2.5, 0`: numbers as Roblox shows them.
fn join(numbers: &[f32]) -> String {
    numbers
        .iter()
        .map(|n| ((n * 1000.0).round() / 1000.0).to_string())
        .collect::<Vec<_>>()
        .join(", ")
}

/// Every number in `text`, ignoring commas, braces and spaces.
fn numbers(text: &str) -> Vec<f32> {
    text.split(|c: char| !(c.is_ascii_digit() || c == '.' || c == '-'))
        .filter_map(|s| s.parse().ok())
        .collect()
}

fn num<T: egui::emath::Numeric>(ui: &mut egui::Ui, n: &mut T, label: &str) -> egui::Response {
    ui.add(
        egui::DragValue::new(n)
            .speed(0.1)
            .max_decimals(3)
            .prefix(label),
    )
}

fn vector2(ui: &mut egui::Ui, v: &mut Vector2) -> bool {
    (num(ui, &mut v.x, "X ") | num(ui, &mut v.y, "Y ")).changed()
}

fn vector3(ui: &mut egui::Ui, v: &mut Vector3) -> bool {
    (num(ui, &mut v.x, "X ") | num(ui, &mut v.y, "Y ") | num(ui, &mut v.z, "Z ")).changed()
}

fn udim(ui: &mut egui::Ui, name: &str, v: &mut UDim) -> bool {
    inline(ui, name, |ui| {
        (num(ui, &mut v.scale, "Scale") | num(ui, &mut v.offset, "Offset")).changed()
    })
}

fn to_orientation(m: Matrix3) -> Vector3 {
    let row = |v: Vector3| Vec3::new(v.x, v.y, v.z);
    let rotation = Mat3::from_cols(row(m.x), row(m.y), row(m.z)).transpose();
    let (y, x, z) = rotation.to_euler(EulerRot::YXZ);
    Vector3::new(x.to_degrees(), y.to_degrees(), z.to_degrees())
}

fn from_orientation(o: Vector3) -> Matrix3 {
    let rows = Mat3::from_euler(
        EulerRot::YXZ,
        o.y.to_radians(),
        o.x.to_radians(),
        o.z.to_radians(),
    )
    .transpose();
    let row = |v: Vec3| Vector3::new(v.x, v.y, v.z);
    Matrix3::new(row(rows.x_axis), row(rows.y_axis), row(rows.z_axis))
}
