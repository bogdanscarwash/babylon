use bevy::prelude::*;

#[test]
fn app_builds_and_updates_headless() {
    let mut app = App::new();
    app.add_plugins(MinimalPlugins);
    app.update();
}
