use fission::prelude::*;
use fission_shell_desktop::DesktopApp;

fn main() -> anyhow::Result<()> {
    let app = DesktopApp::new(InboxApp);
    app.run()
}

#[derive(Default)]
pub struct InboxApp;

impl Widget<AppState> for InboxApp {
    fn build(&self, _ctx: &mut BuildCtx<AppState>, _view: &View<AppState>) -> Node {
        Row::new(vec![
            // Sidebar
            Column::new(vec![
                Text::new("Folders").into(),
                Button::new("Inbox").into(),
                Button::new("Sent").into(),
                Button::new("Trash").into(),
            ])
            .width(200.0)
            .into(),

            // Email List
            Scroll::new(
                Column::new(vec![
                    Text::new("Emails").into(),
                    Button::new("Email 1").into(),
                    Button::new("Email 2").into(),
                    Button::new("Email 3").into(),
                    Button::new("Email 4").into(),
                    Button::new("Email 5").into(),
                    Button::new("Email 6").into(),
                    Button::new("Email 7").into(),
                    Button::new("Email 8").into(),
                    Button::new("Email 9").into(),
                    Button::new("Email 10").into(),
                ])
            )
            .width(300.0)
            .into(),

            // Email Content
            Column::new(vec![
                Text::new("Email Content").into(),
                Text::new("From: ...").into(),
                Text::new("Subject: ...").into(),
                Text::new("Body: ...").into(),
            ])
            .into(),
        ])
        .into()
    }
}

#[derive(Default, Clone, PartialEq)]
pub struct AppState {}

impl fission_core::AppState for AppState {}
