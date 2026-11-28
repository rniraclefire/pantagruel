use rbx_dom_weak::types::Variant;

pub trait DataType: Sized {
    fn from_value(value: &ree::Value) -> Option<Self>;
    fn into_value(self) -> ree::Value;
}

impl DataType for f64 {
    fn from_value(value: &ree::Value) -> Option<Self> {
        match value {
            ree::Value::Number(n) => Some(*n),
            _ => None,
        }
    }

    fn into_value(self) -> ree::Value {
        ree::Value::Number(self)
    }
}

impl DataType for bool {
    fn from_value(value: &ree::Value) -> Option<Self> {
        match value {
            ree::Value::Bool(b) => Some(*b),
            _ => None,
        }
    }

    fn into_value(self) -> ree::Value {
        ree::Value::Bool(self)
    }
}

impl DataType for String {
    fn from_value(value: &ree::Value) -> Option<Self> {
        match value {
            ree::Value::String(s) => Some(s.to_str_lossy().to_string()),
            _ => None,
        }
    }

    fn into_value(self) -> ree::Value {
        ree::Value::String(self.as_str().into())
    }
}

impl DataType for rbx_dom_weak::types::Vector3 {
    fn from_value(value: &ree::Value) -> Option<Self> {
        match value {
            ree::Value::Vector3(v) => Some(*v),
            _ => None,
        }
    }

    fn into_value(self) -> ree::Value {
        ree::Value::Vector3(self)
    }
}

impl DataType for rbx_dom_weak::types::Color3 {
    fn from_value(value: &ree::Value) -> Option<Self> {
        match value {
            ree::Value::Color3(c) => Some(*c),
            _ => None,
        }
    }

    fn into_value(self) -> ree::Value {
        ree::Value::Color3(self)
    }
}

impl DataType for rbx_dom_weak::types::CFrame {
    fn from_value(value: &ree::Value) -> Option<Self> {
        match value {
            ree::Value::Dom(variant) => match &**variant {
                Variant::CFrame(cf) => Some(*cf),
                _ => None,
            },
            _ => None,
        }
    }

    fn into_value(self) -> ree::Value {
        ree::Value::Dom(Variant::CFrame(self).into())
    }
}

pub fn cframe(position: [f32; 3], rows: [[f32; 3]; 3]) -> rbx_dom_weak::types::CFrame {
    let v = |[x, y, z]: [f32; 3]| rbx_dom_weak::types::Vector3::new(x, y, z);
    rbx_dom_weak::types::CFrame::new(
        v(position),
        rbx_dom_weak::types::Matrix3::new(v(rows[0]), v(rows[1]), v(rows[2])),
    )
}

pub fn to_glam(cf: rbx_dom_weak::types::CFrame) -> (glam::Vec3, glam::Mat3) {
    let v = |v: rbx_dom_weak::types::Vector3| glam::Vec3::new(v.x, v.y, v.z);
    let o = cf.orientation;
    (
        v(cf.position),
        glam::Mat3::from_cols(v(o.x), v(o.y), v(o.z)).transpose(),
    )
}
