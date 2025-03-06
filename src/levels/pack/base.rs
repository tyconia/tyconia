use super::*;

pub fn base_mod() -> Pack {
    Pack {
        meta: Meta {
            mod_name: "base".into(),
            version: (0, 0, 0).into(),
        },
        description: "adds automated arm, mover belts and the infinite io machine".into(),
        items: vec!["auto_arm".into(), "mover_belt".into(), "infinite_io".into()],
        research: vec![ResearchId("developer_tools".into())],
        recipes: vec![],
    }
}
