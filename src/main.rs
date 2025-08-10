use bibcitex_ui::{App, utils::observe_app};

fn main() {
    observe_app();

    let index_html = r#"
        <!doctype html>
        <html data-theme="nord">
            <head>
                <title>Dioxus app</title>
                <meta
                    name="viewport"
                    content="width=device-width, initial-scale=1.0, maximum-scale=1.0, user-scalable=no"
                />
            </head>

            <body>
                <div id="main"></div>
            </body>
        </html>
"#.to_string();
    let config = dioxus::desktop::Config::new().with_custom_index(index_html);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(App);
    // dioxus::launch(App);
}
