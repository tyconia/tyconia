![Tyconia hero text](/assets/textures/ui/title_0.png  "Optional title")

# Tyconia
Tyconia is an opinionated game engine built on top of [bevy] for 2D isometric automation and logistics games, providing a structured framework for creating content with declarative-first programming paradigm—using RON and a mature file layout to organize assets and scenarios and a scripting API for more wacky behavior with rhai.

> Depending on your platform, certain [bevy] dependencies are required for e.g. if you're on [linux or wsl].

# Running
* Make sure you have rust
* Start the native app: `cargo run`
* Start the web build: `trunk serve`
    * requires [trunk]: `cargo install --locked trunk`
    * requires `wasm32-unknown-unknown` target: `rustup target add wasm32-unknown-unknown`
    * this will serve your app on `8080` and automatically rebuild + reload it after code changes

# License

This project is licensed under [MPLv2](LICENSE) unless otherwise indicated.

> The Mozilla Public License version 2.0 (MPLv2) is a file-based license, meaning its copyleft provisions apply only to modified files, allowing them to be combined with files under other licenses, including proprietary ones.

# Tailoring your own experience
A typical mod can be as simple as a texture reskin or an entire overhaul. The main game is also a mod by design for ease. Check out the [book]()! The file structure of a mod is as follows:

    mod_name/
    │
    ├── meta.ron         # Metadata: name, version, author, dependencies, description
    │
    ├── assets/              # All mod assets
    │   ├── textures/        # Sprites, UI elements
    │   ├── sounds/          # SFX and music
    │   └── fonts/           # Custom fonts if needed
    │
    ├── definitions/         # Core data definitions
    │   ├── items.ron        # All items introduced/modified
    │   ├── recipes.ron      # Crafting or cooking recipes
    │   ├── research.ron     # Research tree additions
    │   ├── entities.ron     # Machines, NPCs, or interactive objects
    │   └── levels.ron       # Level layouts, if applicable
    │
    ├── story/               # Story beats, dialogue, and narrative
    │   └── beats.ron        # Narrative elements, dialogue, etc.
    │
    ├── scenarios/           # Custom gameplay style
    │   ├── sushi_restaurant.ron  
    │   └── lemon_stand.ron  
    │
    ├── ui/                   # Custom configurations
    │   └── menus.ron        # Custom menus, buttons, etc.
    │
    └── scripts/              # Custom scripting
        └── example.rhai      # Example mod logic (or leave empty if RON-only)

[linux or wsl]: https://github.com/bevyengine/bevy/blob/main/docs/linux_dependencies.md
[bevy]: https://bevyengine.org/
[bevy-learn]: https://bevyengine.org/learn/
[bevy-discord]: https://discord.gg/bevy
[nikl-twitter]: https://twitter.com/nikl_me
[nikl-mastodon]: https://mastodon.online/@nikl_me
[firefox-sound-issue]: https://github.com/NiklasEi/bevy_kira_audio/issues/9
[Bevy Cheat Book]: https://bevy-cheatbook.github.io/introduction.html
[trunk]: https://trunkrs.dev/
[android-instructions]: https://github.com/bevyengine/bevy/blob/latest/examples/README.md#setup
[ios-instructions]: https://github.com/bevyengine/bevy/blob/latest/examples/README.md#setup-1
[mobile_dev_with_bevy_2]: https://www.nikl.me/blog/2023/notes_on_mobile_development_with_bevy_2/
[workflow_bevy_android]: https://www.nikl.me/blog/2023/github_workflow_to_publish_android_app/
[workflow_bevy_ios]: https://www.nikl.me/blog/2023/github_workflow_to_publish_ios_app/
