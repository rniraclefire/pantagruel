use std::{collections::HashMap, sync::LazyLock};

struct PropertyMeta {
    category: Option<String>,
    hidden: bool,
}

static PROPERTY_META: LazyLock<HashMap<(String, String), PropertyMeta>> = LazyLock::new(load);

fn load() -> HashMap<(String, String), PropertyMeta> {
    let dump: serde_json::Value = serde_json::from_slice(ree::data::REFLECTION).unwrap();
    let mut map = HashMap::new();
    for class in dump["classes"].as_array().unwrap() {
        let class_name = class["name"].as_str().unwrap();
        for p in class["properties"].as_array().unwrap() {
            let tags = p["tags"].as_array();
            let has_tag = |t: &str| tags.is_some_and(|tags| tags.iter().any(|x| x == t));
            map.insert(
                (
                    class_name.to_owned(),
                    p["name"].as_str().unwrap().to_owned(),
                ),
                PropertyMeta {
                    category: p["display"]["Category"].as_str().map(str::to_owned),
                    hidden: has_tag("Deprecated") || has_tag("Hidden"),
                },
            );
        }
    }
    map
}

pub fn category_of(class: &str, property: &str) -> &'static str {
    PROPERTY_META
        .get(&(class.to_owned(), property.to_owned()))
        .and_then(|m| m.category.as_deref())
        .unwrap_or("Other")
}

pub fn is_hidden(class: &str, property: &str) -> bool {
    PROPERTY_META
        .get(&(class.to_owned(), property.to_owned()))
        .is_some_and(|m| m.hidden)
}
