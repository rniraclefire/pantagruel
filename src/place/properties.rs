use glam::{Mat3, Vec3};
use rbx_dom_weak::types::{CFrame, Matrix3, Variant, Vector3};

use super::Place;

impl Place {
    fn with_property<T>(
        &self,
        id: ree::InstanceId,
        property: &str,
        f: impl FnOnce(&ree::Value) -> Option<T>,
    ) -> Option<T> {
        let engine = &self.emulator.engine;
        let name = engine.reflection.name_id(property)?;
        f(engine.state.borrow().dom.raw_get(id, name)?)
    }

    pub fn get(&self, id: ree::InstanceId, property: &str) -> Option<ree::Value> {
        self.with_property(id, property, |v| Some(v.clone()))
    }

    pub fn set(&self, id: ree::InstanceId, name: ree::NameId, value: ree::Value) -> bool {
        let engine = &self.emulator.engine;
        let mut state = engine.state.borrow_mut();
        ree::instance::raw_set(
            &mut state,
            &engine.reflection,
            &engine.hooks.borrow(),
            id,
            name,
            value,
        )
    }

    pub fn get_number(&self, id: ree::InstanceId, property: &str) -> Option<f64> {
        self.with_property(id, property, |v| v.as_number())
    }

    pub fn get_cframe(&self, id: ree::InstanceId, property: &str) -> Option<CFrame> {
        self.with_property(id, property, |v| match as_variant(v)? {
            Variant::CFrame(cf) => Some(*cf),
            _ => None,
        })
    }
}

fn as_variant(value: &ree::Value) -> Option<&Variant> {
    match value {
        ree::Value::Dom(variant) => Some(variant),
        _ => None,
    }
}

fn vec3(v: Vector3) -> Vec3 {
    Vec3::new(v.x, v.y, v.z)
}

pub fn cframe(position: [f32; 3], rows: [[f32; 3]; 3]) -> CFrame {
    let v = |[x, y, z]: [f32; 3]| Vector3::new(x, y, z);
    CFrame::new(
        v(position),
        Matrix3::new(v(rows[0]), v(rows[1]), v(rows[2])),
    )
}

pub fn to_glam(cf: CFrame) -> (Vec3, Mat3) {
    let o = cf.orientation;
    let rotation = Mat3::from_cols(vec3(o.x), vec3(o.y), vec3(o.z)).transpose();
    (vec3(cf.position), rotation)
}
