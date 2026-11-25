use std::rc::Rc;

use glam::{Mat3, Vec3};
use rbx_dom_weak::types::{CFrame, Matrix3, Variant, Vector3};

use super::Place;

impl Place {
    pub fn get(&self, id: ree::InstanceId, property: &str) -> Option<ree::Value> {
        let engine = &self.emulator.engine;
        let name = engine.reflection.name_id(property)?;
        engine.state.borrow().dom.raw_get(id, name).cloned()
    }

    pub fn get_number(&self, id: ree::InstanceId, property: &str) -> Option<f64> {
        self.get(id, property)?.as_number()
    }

    pub fn get_cframe(&self, id: ree::InstanceId, property: &str) -> Option<CFrame> {
        match self.get(id, property)? {
            ree::Value::Dom(variant) => match &*variant {
                Variant::CFrame(cframe) => Some(*cframe),
                _ => None,
            },
            _ => None,
        }
    }
}

pub fn cframe(position: [f32; 3], rows: [[f32; 3]; 3]) -> CFrame {
    let vector = |v: [f32; 3]| Vector3::new(v[0], v[1], v[2]);
    CFrame::new(
        vector(position),
        Matrix3::new(vector(rows[0]), vector(rows[1]), vector(rows[2])),
    )
}

pub fn to_glam(cf: CFrame) -> (Vec3, Mat3) {
    let p = cf.position;
    let o = cf.orientation;
    (
        Vec3::new(p.x, p.y, p.z),
        Mat3::from_cols(
            Vec3::new(o.x.x, o.y.x, o.z.x),
            Vec3::new(o.x.y, o.y.y, o.z.y),
            Vec3::new(o.x.z, o.y.z, o.z.z),
        ),
    )
}
