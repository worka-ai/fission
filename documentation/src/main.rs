mod charts;
mod components;

use anyhow::Result;
use components::{DocsFooter, DocsState, RoutedHomePage};
use fission::prelude::*;
use fission_shell_site::{build_from_cli, FissionSite};

fn main() -> Result<()> {
    build_from_cli(site_app())
}

fn site_app() -> FissionSite {
    FissionSite::new()
        .light_dark_themes(Theme::default(), Theme::dark(), DesignMode::Dark)
        .route_widget::<DocsState, _>(
            "/",
            "Fission",
            Some(
                "Build production desktop, web, Android, and iOS apps with one Rust UI framework."
                    .to_string(),
            ),
            RoutedHomePage::new("/"),
        )
        .footer_widget::<DocsState, _>(DocsFooter)
        .user_css(include_str!("../site/overrides.css"))
        .content_transform(charts::expand_documentation_mdx)
}
