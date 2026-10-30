use bevy::prelude::*;
use std::io::{self, Read, Write};
use std::thread;
use std::time::Duration;
use tyconia::*;

fn main() {
    println!("╔══════════════════════════════════════════╗");
    println!("║       Tyconia Story Simulator           ║");
    println!("╚══════════════════════════════════════════╝");
    println!();

    // List stories
    let stories_dir = "assets/mods/tyconic/stories";
    let mut story_files = vec![];
    if let Ok(entries) = std::fs::read_dir(stories_dir) {
        for entry in entries.flatten() {
            if entry.path().extension().and_then(|s| s.to_str()) == Some("ron") {
                if let Some(name) = entry.file_name().to_str() {
                    story_files.push(name.to_string());
                }
            }
        }
    }
    story_files.sort();

    if story_files.is_empty() {
        println!("No story files found in {}", stories_dir);
        return;
    }

    println!("Available stories:");
    for (i, s) in story_files.iter().enumerate() {
        println!("  {}. {}", i + 1, s);
    }
    println!();
    print!("Select story number: ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    if let Err(_) = io::stdin().read_to_string(&mut input) {
        return;
    }
    let selected: usize = match input.trim().parse() {
        Ok(n) if n > 0 && n <= story_files.len() => n - 1,
        _ => return,
    };

    let story_path = format!("{}/{}", stories_dir, story_files[selected]);
    println!("\nLoading: {}", story_path);

    let content = match std::fs::read_to_string(&story_path) {
        Ok(c) => c,
        Err(e) => {
            println!("Failed to read: {}", e);
            return;
        }
    };

    // Try to deserialize as ScriptDeclared wrapped in Bevy reflect format
    // The files are in Bevy Reflect + RON format like other assets
    let mut app = App::new();
    app.register_type::<ScriptDeclared>()
        .register_type::<ScriptAction>()
        .register_type::<Conversation>();

    // For now, try to parse as RON with type info
    // Bevy's reflect deserialization is complex - but let's do a simple approach
    // by using ron to deserialize the structure
    let script: ScriptDeclared = match ron::de::from_str(&content) {
        Ok(s) => s,
        Err(e) => {
            println!("Failed to deserialize (try direct): {}", e);
            // Try alternative parsing
            return;
        }
    };

    simulate_story(&script);
}

fn simulate_story(script: &ScriptDeclared) {
    println!("\n═══════════════════════════════════════════");
    println!("Story: {}", script.label);
    println!("Total actions: {}", script.actions.len());
    println!("═══════════════════════════════════════════");
    println!("\nPress Enter to begin simulation...");
    let _ = io::stdin().read_line(&mut String::new());

    let mut step = 0;
    execute_actions(&script.actions, &mut step, script.actions.len(), 0);
    
    println!("\n═══════════════════════════════════════════");
    println!("Story Simulation Complete!");
    println!("═══════════════════════════════════════════");
}

fn execute_actions(actions: &[ScriptAction], step: &mut usize, total: usize, depth: usize) {
    let indent = "  ".repeat(depth);
    for action in actions {
        *step += 1;
        println!("\n{}───────────────────────────────────────────", indent);
        println!("{}[Step {}/{}]", indent, step, total);
        println!("{}{}", indent, "=".repeat(40 - depth * 2));

        match action {
            ScriptAction::SpawnToLevel { level_label } => {
                println!("{}🗺️  [SPAWN LEVEL] {}", indent, level_label);
                println!("{}   Loading environment...", indent);
                thread::sleep(Duration::from_millis(800));
            }
            ScriptAction::CameraMove { camera, position } => {
                println!("{}📷 [CAMERA_MOVE] {} → {:?}", indent, camera, position);
                thread::sleep(Duration::from_millis(400));
            }
            ScriptAction::CameraLookAt { camera, target } => {
                println!("{}📷 [CAMERA_LOOKAT] {} looking at {:?}", indent, camera, target);
                thread::sleep(Duration::from_millis(400));
            }
            ScriptAction::CameraZoom { camera, zoom } => {
                println!("{}📷 [CAMERA_ZOOM] {} zoom = {}", indent, camera, zoom);
                thread::sleep(Duration::from_millis(400));
            }
            ScriptAction::CameraFade { camera, duration, from, to } => {
                println!("{}🎬 [CAMERA_FADE] {} {} → {} over {}s", indent, camera, from, to, duration);
                simulate_delay(*duration);
            }
            ScriptAction::Move { character, reposition } => {
                println!("{}🚶 [MOVE] {} moves to {:?}", indent, character, reposition);
                simulate_delay(0.6);
            }
            ScriptAction::Converse { character, content } => match content {
                Conversation::Continue { text } => {
                    println!("{}💬 {}:", indent, character);
                    println!("{}   \"{}\"", indent, text);
                    println!();
                    print!("{}   [Press Enter to continue] ", indent);
                    io::stdout().flush().unwrap();
                    let _ = io::stdin().read_line(&mut String::new());
                    println!();
                }
                Conversation::Prompt { text, choices } => {
                    println!("{}💬 {}:", indent, character);
                    println!("{}   \"{}\"", indent, text);
                    println!();
                    let choices_vec: Vec<String> = choices.keys().cloned().collect();
                    for (i, choice_text) in choices_vec.iter().enumerate() {
                        println!("{}   {}. {}", indent, i + 1, choice_text);
                    }
                    println!();
                    loop {
                        print!("{}   Choose an option: ", indent);
                        io::stdout().flush().unwrap();
                        let mut input = String::new();
                        if io::stdin().read_line(&mut input).is_err() {
                            break;
                        }
                        if let Ok(num) = input.trim().parse::<usize>() {
                            if num >= 1 && num <= choices_vec.len() {
                                println!();
                                println!("{}   ➤ You chose: {}", indent, choices_vec[num - 1]);
                                thread::sleep(Duration::from_millis(400));
                                break;
                            }
                        }
                        println!("{}   Invalid choice, try again.", indent);
                    }
                    println!();
                }
            },
            ScriptAction::Wait { duration } => {
                println!("{}⏳ [WAIT] {} seconds...", indent, duration);
                simulate_delay(*duration);
            }
            ScriptAction::WaitUntil { condition } => {
                println!("{}⏸️  [WAIT_UNTIL] Condition: {:?}", indent, condition);
                let chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                for i in 0..12 {
                    print!("\r{}   {} Evaluating...", indent, chars[i % chars.len()]);
                    io::stdout().flush().unwrap();
                    thread::sleep(Duration::from_millis(100));
                }
                print!("\r{}   ✓ Condition satisfied\n", indent);
                thread::sleep(Duration::from_millis(200));
            }
            ScriptAction::Parallel { actions } => {
                println!("{}⚡ [PARALLEL] {} actions executing simultaneously", indent, actions.len());
                for (i, a) in actions.iter().enumerate() {
                    println!("{}   ├─ {}: {:?}", indent, i + 1, a);
                }
                println!("{}   Simulating parallel execution...", indent);
                thread::sleep(Duration::from_millis(600));
            }
            ScriptAction::Sequence { actions } => {
                println!("{}🔗 [SEQUENCE] {} actions in sequence", indent, actions.len());
                thread::sleep(Duration::from_millis(200));
                execute_actions(actions, step, *step + actions.len(), depth + 1);
            }
            ScriptAction::ArrowIndications { targets } => {
                println!("{}➡️  [ARROWS] Showing {} indicator(s)", indent, targets.len());
                for t in targets {
                    println!("{}     → {:?}", indent, t);
                }
                thread::sleep(Duration::from_millis(400));
            }
            ScriptAction::HighlightIndications { targets } => {
                println!("{}✨ [HIGHLIGHT] Highlighting {} tile(s)", indent, targets.len());
                thread::sleep(Duration::from_millis(400));
            }
            ScriptAction::AlterValue { resource, value } => {
                println!("{}💰 [ALTER] {} {} {}", indent, resource, if *value >= 0.0 { "+" } else { "" }, value);
                thread::sleep(Duration::from_millis(400));
            }
            ScriptAction::AwaitResearch { research_id } => {
                println!("{}🔬 [AWAIT_RESEARCH] {}", indent, research_id);
                let chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
                for i in 0..18 {
                    print!("\r{}   {} Research in progress...", indent, chars[i % chars.len()]);
                    io::stdout().flush().unwrap();
                    thread::sleep(Duration::from_millis(80));
                }
                print!("\r{}   ✓ Research unlocked\n", indent);
                thread::sleep(Duration::from_millis(300));
            }
            ScriptAction::JumpTo { condition, story_beat_label } => {
                if let Some(c) = condition {
                    println!("{}🔀 [JUMP] → '{}' (if {:?})", indent, story_beat_label, c);
                } else {
                    println!("{}🔀 [JUMP] → '{}'", indent, story_beat_label);
                }
                thread::sleep(Duration::from_millis(400));
            }
        }
    }
}

fn simulate_delay(duration: f32) {
    if duration <= 0.0 {
        return;
    }
    let total_ms = (duration * 1000.0) as u64;
    let chars = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let start = std::time::Instant::now();
    let mut i = 0;
    while start.elapsed().as_millis() < total_ms as u128 {
        let elapsed = start.elapsed().as_millis() as f32 / 1000.0;
        print!("\r   {} {:.1}s / {:.1}s", chars[i % chars.len()], elapsed, duration);
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(80));
        i += 1;
    }
    print!("\r   ✓ Completed in {:.1}s\n", duration);
    thread::sleep(Duration::from_millis(150));
}
