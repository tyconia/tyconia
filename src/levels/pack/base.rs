use super::*;

pub fn base_mod() -> Pack {
    Pack {
        meta: Meta {
            mod_name: "base".into(),
            namespace: namespace::Namespace::Vanilla,
            version: (0, 0, 0).into(),
        },
        description: "adds automated arm, mover belts and the infinite io machine".into(),
        items: vec![
            ItemId("auto_arm".into()),
            ItemId("mover_belt".into()),
            ItemId("infinite_io".into()),
        ],
        research: vec![ResearchId("developer_tools".into())],
        recipes: vec![],
    }
}
