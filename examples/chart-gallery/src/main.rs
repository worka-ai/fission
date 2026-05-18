mod app;
mod charts;
mod data;
mod showcase;
mod state;
mod style;

use app::GalleryApp;
use fission_shell_desktop::DesktopApp;
use state::GalleryState;

fn main() -> anyhow::Result<()> {
    let app = DesktopApp::new(GalleryApp)
        .with_title("Fission Chart Gallery")
        .with_sync_env(|state: &GalleryState, env: &mut fission_core::Env| {
            env.theme = if state.dark_theme {
                fission_theme::Theme::dark()
            } else {
                fission_theme::Theme::default()
            };
        });

    app.run()
}
