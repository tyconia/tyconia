fn on_script_loaded() {
  print("script is loaded")

}

fn on_event(arg1) {
  print(`player max speed is ${arg1.max_speed}`);
  print(`player acceleration is ${arg1.acceleration}`);
}

fn on_script_unloaded() {
  print("script is unloaded");
}

