use bevy::prelude::*;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;
use tyconia::*;

fn main() {
    println!("╔══════════════════════════════════════════╗");
    println!("║      Tyconia Interactive Story Player   ║");
    println!("╚══════════════════════════════════════════╝");
    println!();
    println!("Loading comprehensive_demo story...");
    println!();

    let script = ScriptDeclared {
        label: "comprehensive_demo".into(),
        actions: vec![
            ScriptAction::SpawnToLevel {
                level_label: "prologue_restaurant_0".into(),
            },
            ScriptAction::CameraMove {
                camera: "main".into(),
                position: Vec2::ZERO,
            },
            ScriptAction::CameraZoom {
                camera: "main".into(),
                zoom: 1.0,
            },
            ScriptAction::CameraFade {
                camera: "main".into(),
                duration: 1.0,
                from: 0.0,
                to: 1.0,
            },
            ScriptAction::Parallel {
                actions: vec![
                    ScriptAction::movement("base::pov_player".into(), Vec2::ZERO),
                    ScriptAction::Sequence {
                        actions: vec![
                            ScriptAction::wait(1.0),
                            ScriptAction::movement(
                                "base::boss_frank".into(),
                                Vec2::new(-8.0, -4.0),
                            ),
                            ScriptAction::converse(
                                "bois_of_liberty::frank".into(),
                                "Cousin! Been expecting ya.".into(),
                            ),
                        ],
                    },
                ],
            },
            ScriptAction::ArrowIndications {
                targets: vec![Vec2::new(-8.0, -4.0), Vec2::new(0.0, 0.0)],
            },
            ScriptAction::HighlightIndications {
                targets: vec![10, 11, 12],
            },
            ScriptAction::converse(
                "base::pov_player".into(),
                "Frank... I thought we had a deal.".into(),
            ),
            ScriptAction::choice(
                "bois_of_liberty::frank".into(),
                "Aye, where's my cut, cuh?".to_string(),
                [
                    (
                        "naw, boss I need more time".to_string(),
                        "bois_of_liberty::determined_frank".to_string(),
                    ),
                    (
                        "Here, take what I have".to_string(),
                        "bois_of_liberty::pleased_frank".to_string(),
                    ),
                ],
            ),
            ScriptAction::AlterValue {
                resource: "money".into(),
                value: -500.0,
            },
            ScriptAction::AwaitResearch {
                research_id: "diner_expansion_1".into(),
            },
            ScriptAction::WaitUntil {
                condition: ConditionFlag::ReachedMetric {
                    metric: "customer_satisfaction".into(),
                    value: 75.0,
                },
            },
            ScriptAction::CameraLookAt {
                camera: "main".into(),
                target: Vec2::new(-8.0, -4.0),
            },
            ScriptAction::CameraFade {
                camera: "main".into(),
                duration: 0.5,
                from: 1.0,
                to: 0.0,
            },
            ScriptAction::JumpTo {
                condition: None,
                story_beat_label: "chapter_1".into(),
            },
        ],
    };

    println!("Story: {}", script.label);
    println!("Total actions: {}", script.actions.len());
    println!();
    println!("Press Enter to start...");
    let _ = io::stdin().read_line(&mut String::new());

    let mut step = 0;
    for action in &script.actions {
        step += 1;
        println!("───────────────────────────────────────────");
        println!("[Step {}/{}]", step, script.actions.len());
        println!();

        match action {
            ScriptAction::SpawnToLevel { level_label } => {
                println!("🗺️  Loading level: {}", level_label);
                thread::sleep(Duration::from_millis(800));
            }
            ScriptAction::CameraMove { camera, position } => {
                println!("📷 Camera [{}] moving to {:?}", camera, position);
                thread::sleep(Duration::from_millis(400));
            }
            ScriptAction::CameraZoom { camera, zoom } => {
                println!("📷 Camera [{}] zooming to {}", camera, zoom);
                thread::sleep(Duration::from_millis(400));
            }
            ScriptAction::CameraLookAt { camera, target } => {
                println!("📷 Camera [{}] looking at {:?}", camera, target);
                thread::sleep(Duration::from_millis(400));
            }
            ScriptAction::CameraFade { camera, duration, from, to } => {
                println!(
                    "🎬 Camera [{}] fading from {} to {} over {}s",
                    camera, from, to, duration
                );
                thread::sleep(Duration::from_millis((duration * 1000.0) as u64));
            }
            ScriptAction::Move { character, reposition } => {
                println!("🚶 {} moves to {:?}", character, reposition);
                thread::sleep(Duration::from_millis(600));
            }
            ScriptAction::Converse { character, content } => match content {
                Conversation::Continue { text } => {
                    println!("💬 {}:", character);
                    println!("   \"{}\"", text);
                    println!();
                    print!("   [Press Enter to continue]");
                    io::stdout().flush().unwrap();
                    let _ = io::stdin().read_line(&mut String::new());
                    println!();
                }
                Conversation::Prompt { text, choices } => {
                    println!("💬 {}:", character);
                    println!("   \"{}\"", text);
                    println!();
                    let choices_vec: Vec<(String, String)> = choices.iter().map(|(k,v)| (k.clone(), v.clone())).collect();
                    for (i, (choice_text, _)) in choices_vec.iter().enumerate() {
                        println!("   {}. {}", i + 1, choice_text);
                    }
                    println!();
                    loop {
                        print!("   Choose an option: ");
                        io::stdout().flush().unwrap();
                        let mut input = String::new();
                        if io::stdin().read_line(&mut input).is_err() {
                            break;
                        }
                        if let Ok(num) = input.trim().parse::<usize>() {
                            if num >= 1 && num <= choices_vec.len() {
                                println!();
                                println!("   ➤ You chose: {}", choices_vec[num - 1].0);
                                thread::sleep(Duration::from_millis(400));
                                break;
                            }
                        }
                        println!("   Invalid choice, try again.");
                    }
                    println!();
                }
            },
            ScriptAction::Wait { duration } => {
                println!("⏳ Waiting for {} seconds...", duration);
                let total_ms = (*duration * 1000.0) as u64;
                let chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                let start = std::time::Instant::now();
                let mut i = 0;
                while start.elapsed().as_millis() < total_ms as u128 {
                    let elapsed = start.elapsed().as_millis() as f32 / 1000.0;
                    print!("\r   {} {:.1}s", chars[i % chars.len()], elapsed);
                    io::stdout().flush().unwrap();
                    thread::sleep(Duration::from_millis(80));
                    i += 1;
                }
                print!("\r   ✓ Done in {:.1}s\n", duration);
                thread::sleep(Duration::from_millis(200));
            }
            ScriptAction::WaitUntil { condition } => {
                println!("⏸️  Waiting until condition met: {:?}", condition);
                let chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                for i in 0..15 {
                    print!("\r   {} Checking...", chars[i % chars.len()]);
                    io::stdout().flush().unwrap();
                    thread::sleep(Duration::from_millis(100));
                }
                print!("\r   ✓ Condition satisfied\n");
                thread::sleep(Duration::from_millis(200));
            }
            ScriptAction::Parallel { actions } => {
                println!("⚡ PARALLEL - {} actions simultaneously:", actions.len());
                for a in actions {
                    println!("   • {:?}", a);
                }
                thread::sleep(Duration::from_millis(800));
            }
            ScriptAction::Sequence { actions } => {
                println!("🔗 SEQUENCE - {} actions in order:", actions.len());
                for a in actions {
                    println!("   • {:?}", a);
                }
                thread::sleep(Duration::from_millis(800));
            }
            ScriptAction::ArrowIndications { targets } => {
                println!("➡️  Showing arrows at {} locations", targets.len());
                thread::sleep(Duration::from_millis(400));
            }
            ScriptAction::HighlightIndications { targets } => {
                println!("✨ Highlighting {} tiles", targets.len());
                thread::sleep(Duration::from_millis(400));
            }
            ScriptAction::AlterValue { resource, value } => {
                println!("💰 Altering {} by {}", resource, value);
                thread::sleep(Duration::from_millis(400));
            }
            ScriptAction::AwaitResearch { research_id } => {
                println!("🔬 Awaiting research: {}", research_id);
                let chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                for i in 0..20 {
                    print!("\r   {} Researching...", chars[i % chars.len()]);
                    io::stdout().flush().unwrap();
                    thread::sleep(Duration::from_millis(80));
                }
                print!("\r   ✓ Research complete\n");
                thread::sleep(Duration::from_millis(300));
            }
            ScriptAction::JumpTo {
                condition,
                story_beat_label,
            } => {
                if let Some(c) = condition {
                    println!("🔀 Jumping to '{}' if {:?}", story_beat_label, c);
                } else {
                    println!("🔀 Jumping to: {}", story_beat_label);
                }
                thread::sleep(Duration::from_millis(400));
            }
        }
        println!();
    }

    println!("═══════════════════════════════════════════");
    println!("Story Complete!");
    println!("═══════════════════════════════════════════");
}
