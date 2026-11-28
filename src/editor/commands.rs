use crate::editor::EditorState;

pub enum Command {
    Select(Vec<ree::InstanceId>),
    SetProperty {
        id: ree::InstanceId,
        name: ree::NameId,
        value: ree::Value,
    },
    Rename {
        id: ree::InstanceId,
        name: String,
    },
    Insert {
        parent: ree::InstanceId,
        class: String,
    },
    Delete(Vec<ree::InstanceId>),
}

impl Command {
    pub fn run(self, editor: &mut EditorState) -> anyhow::Result<()> {
        match self {
            Self::Select(ids) => editor.selection = ids,
            Self::SetProperty { id, name, value } => {
                editor.place.set(id, name, value);
            }
            Self::Rename { id, name } => editor.place.rename(id, &name)?,
            Self::Insert { parent, class } => {
                let id = editor.place.insert(parent, &class)?;
                editor.selection = vec![id];
            }
            Self::Delete(ids) => {
                editor.place.delete(&ids)?;
                editor.selection.retain(|id| !ids.contains(id));
            }
        }

        Ok(())
    }
}
