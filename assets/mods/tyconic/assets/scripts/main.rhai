fn on_script_loaded() {
  Log.info.call("tyconic is loaded");

  // Surfaces.trigger_transition_surface.call();
  // Surfaces.trigger_resurface.call();
  // Surfaces.add_surface.call("Sup");
}

fn on_event(arg1, arg2) {
 // print(`player acceleration is ${arg1.acceleration}`);
 // Log.info.call(`player max speed is ${arg1.max_speed}`);
  let name = "brickie";

  // Log.info.call(`The ${name}'s acceleration is ${arg1.acceleration}`);
  // Log.info.call(`The other name is ${arg2}`);
  // Surfaces.create.call();
  let num_1 = 10;
  let num_2 = 10;
  // let sum = Surfaces.add_to.call(num_1, num_2);
  // Log.info.call(`sum of ${num_1} and ${num_2} is ${sum}`);
  // let game_state = Surfaces.builder.call();
  // Log.info.call(`Game state is ${game_state.index}`);
}

fn on_script_unloaded() {
  Log.info.call("tyconic is unloaded. Thank you for playing my game :>");
}

fn on_surface_create(create_surface) {
  Log.info.call("We are about to create surfaces :>");
}

// called at world creation menu aka "New Game" menu.
fn on_world_configure() {
  let starting_money = Definitions.get("starting_money");
}

fn on_world_exit() {}

fn tutorial() {
  // Log.info.call(`${num_1} + ${num_2} = ${num_1 + num_2}`);
  Log.info.call("This is tutorial");
}

fn on_level_start(level, string_thing) {
  // Log.info.call(`The string thing is "${string_thing}"`);
  // Log.info.call(`Current level is "${level.current}"`);
  // level.create.call();

  // LevelManager.create.call();
  // LevelManager.register.call("tutorial");
}


