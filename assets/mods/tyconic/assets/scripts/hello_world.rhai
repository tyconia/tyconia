// level is the root level where you can spawn levels
// that may also have children levels
fn on_world_start(level, transition_ctx) {
  // Insert a basic resource
  Level.insert_resource("money", 0);
  
  
  // Insert a configurable resource with a name, default, and description
  //Level.configure_resource("money", starting_money, {
  //  display_name: "Starting Capital",
  //  description: "Initial funds available at the start.",
  //  min: 0.0,
  //  max: 10000.0,
  //  step: 50.0, // Optional UI increment
  //});

  // Disable infinite inventory mode
  Level.set_item_catalog_enabled(false);

  // Set player's starting inventory in bulk
  Player.set_inventory([
    { slot: 0, item: "base::auto_arm" },
    { slot: 1, item: "tyconic::red_table" }
  ])
  .to("jaime#2456");

  // Alternative: Append items dynamically
  Player.add_to_inventory(2, "base::infinite_io");





  // loading screen is the first to appear before anything
  // basically `transition_ctx.capture_loading();`
  // controls are also captured `level.capture_pov_player_ctrl();`;
  
  // transitioning to a level don't remove the loading screen
  // until transition_ctx.release_loading()
  level.register("urban_overworld", |level, transition_ctx| {

    // generate the neighborhood scene that includes 
    // the lemonade stand, your first business
    level.load_scene("neighborhood_0", { x: 0., y: -10. }) or {
      // transitions to a builtin fatal error screen
      // with a button to quit to menu
      transition_ctx.fatal_error("failed to load neighborhood scene");
    };

    // all non-operator players that has no other special roles or privileges
    // are collectively referred to as "pov_player"
    // for e.g. here, all non-operator players are forced to spawn at this location
    level.spawn_pov_player({ x: 20., y: 10. });

    // up to this point, the loading screen is in place.
    transition_ctx.release_loading();

    // player can now move and interact with world
    level.release_pov_player_ctrl();

    // other players aren't shown until here. Useful to keep
    // tutorials and showcases without the chaos
    level.show_rest_of_players();

    // set event handlers for player achievement
    // non_operators = true is default anyway
    level.players(non_operators = true).for_each(|player| {
      player.on_achievement("tyconic::lemon_grabbed_profits", |level| {
        // in essence basically every player gets their own restaurant lol
        level.transition("second_business", {
          owner: player,
          visitable: false,
        });
      });
    });

    level.register("second_business", |level, transition_ctx| {
      level.load_scene("starter_restaurant", { x: 0., y: 0. }) or {
        // transitions to a builtin fatal error screen
        // with a button to quit to menu
        transition_ctx.fatal_error("failed to load starter restaurant scene");
      };
    });

  });

  level.transition("urban_overworld");
}
