use bevy::prelude::*;
use tyconia::*;

fn main() {
    App::new()
        .add_plugins(MinimalPlugins)
        .add_plugins(StoryPlugin)
        .add_systems(Startup, load_story)
        .add_systems(Update, |reviewer: Res<HeadlessStoryReviewer>| {
            if reviewer.active_scripts.is_empty() { return; }
            eprintln!("Active scripts: {}", reviewer.active_scripts.len());
            for s in &reviewer.active_scripts {
                eprintln!("  {}: {:?} [{}/{}]", s.label, s.status, s.current_action_index, s.actions.len());
            }
        })
        .run();
}

fn load_story(mut reviewer: ResMut<HeadlessStoryReviewer>) {
    eprintln!("LOADING STORY...");
    let script = ScriptDeclared {
        label: "comprehensive_demo".into(),
        actions: vec![
            ScriptAction::SpawnToLevel { level_label: "prologue_restaurant_0".into() },
            ScriptAction::CameraMove { camera: "main".into(), position: Vec2::ZERO },
            ScriptAction::Parallel {
                actions: vec![
                    ScriptAction::movement("base::pov_player".into(), Vec2::ZERO),
                    ScriptAction::Sequence {
                        actions: vec![
                            ScriptAction::wait(0.1),
                            ScriptAction::movement("base::boss_frank".into(), Vec2::new(-8.0, -4.0)),
                            ScriptAction::converse("bois_of_liberty::frank".into(), "Cousin! Been expecting ya.".into()),
                        ]
                    }
                ]
            },
            ScriptAction::converse("base::pov_player".into(), "Frank... I thought we had a deal.".into()),
            ScriptAction::JumpTo { condition: None, story_beat_label: "chapter_1".into() },
        ],
    };

    reviewer.active_scripts.push(ActiveScript {
        label: script.label,
        current_action_index: 0,
        status: ScriptStatus::Playing,
        actions: script.actions,
        elapsed_in_action: 0.0,
        step_count: 0,
    });
}
