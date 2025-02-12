fn on_script_loaded() {
  Base.log_info.call("tyconic is loaded");

  // Surfaces.trigger_transition_surface.call();
  // Surfaces.trigger_resurface.call();
  Surfaces.add_surface.call("Sup");
}

fn on_event(arg1) {
 // print(`player max speed is ${arg1.max_speed}`);
 // print(`player acceleration is ${arg1.acceleration}`);
  // let name = "qat";
  // print(`the name is ${name}`);
  // Base.hello_world.call("woof");
  // log_error(name);
}

fn on_script_unloaded() {
  Base.log_info.call("tyconic is unloaded. Thank you for playing my game :>");
}

fn on_surface_create(create_surface) {
  Base.log_info.call("We are about to create surfaces :>");
}
