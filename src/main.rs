use bibcitex_ui::App;

fn main() {
    let index_html = include_str!("../index.html").to_string();
    let config = dioxus::desktop::Config::new().with_custom_index(index_html);
    dioxus::LaunchBuilder::desktop()
        .with_cfg(config)
        .launch(App);
    // dioxus::launch(App);
}
