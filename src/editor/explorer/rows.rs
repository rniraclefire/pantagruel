use std::collections::HashSet;

pub struct Row {
    pub id: ree::InstanceId,
    pub depth: usize,
    pub has_children: bool,
}

pub fn flatten_rows(
    dom: &ree::Dom,
    roots: impl IntoIterator<Item = ree::InstanceId>,
    expanded: &HashSet<ree::InstanceId>,
    filter: Option<&HashSet<ree::InstanceId>>,
) -> Vec<Row> {
    let mut rows = Vec::new();
    for root in roots {
        push_rows(dom, root, 0, expanded, filter, &mut rows);
    }
    rows
}

fn push_rows(
    dom: &ree::Dom,
    id: ree::InstanceId,
    depth: usize,
    expanded: &HashSet<ree::InstanceId>,
    filter: Option<&HashSet<ree::InstanceId>>,
    rows: &mut Vec<Row>,
) {
    if filter.is_some_and(|keep| !keep.contains(&id)) {
        return;
    }

    let children = dom.children_of(id);
    rows.push(Row {
        id,
        depth,
        has_children: !children.is_empty(),
    });

    if filter.is_some() || expanded.contains(&id) {
        for &child in children {
            push_rows(dom, child, depth + 1, expanded, filter, rows);
        }
    }
}

pub fn matching_rows(
    dom: &ree::Dom,
    root: ree::InstanceId,
    mut matches: impl FnMut(&str) -> bool,
) -> HashSet<ree::InstanceId> {
    let mut keep = HashSet::new();

    for id in dom.descendants(root) {
        let Some(node) = dom.get(id) else { continue };
        if !matches(&node.name.to_str_lossy()) {
            continue;
        }

        let mut current = Some(id);
        while let Some(ancestor) = current {
            if ancestor == root || !keep.insert(ancestor) {
                break;
            }
            current = dom.parent_of(ancestor);
        }
    }

    keep
}
