use std::rc::Rc;

use rbx_dom_weak::types::{CFrame, Matrix3, Variant, Vector3};

use super::Place;

impl Place {
    pub fn get(&self, id: ree::InstanceId, property: &str) -> Option<ree::Value> {
        let engine = &self.emulator.engine;
        let name = engine.reflection.name_id(property)?;
        engine.state.borrow().dom.raw_get(id, name).cloned()
    }

    pub fn set(&mut self, id: ree::InstanceId, property: &str, value: ree::Value) -> bool {
        let engine = &self.emulator.engine;
        let Some(name) = engine.reflection.name_id(property) else {
            return false;
        };
        let hooks = engine.hooks.borrow();
        let mut state = engine.state.borrow_mut();
        let changed =
            ree::instance::raw_set(&mut state, &engine.reflection, &hooks, id, name, value);
        if changed && property == "Name" {
            state.dom.refresh_name(&engine.reflection, id);
        }
        changed
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

    pub fn set_cframe(&mut self, id: ree::InstanceId, property: &str, cframe: CFrame) -> bool {
        self.set(
            id,
            property,
            ree::Value::Dom(Rc::new(Variant::CFrame(cframe))),
        )
    }
}

pub fn cframe(position: [f32; 3], rows: [[f32; 3]; 3]) -> CFrame {
    let vector = |v: [f32; 3]| Vector3::new(v[0], v[1], v[2]);
    CFrame::new(
        vector(position),
        Matrix3::new(vector(rows[0]), vector(rows[1]), vector(rows[2])),
    )
}
