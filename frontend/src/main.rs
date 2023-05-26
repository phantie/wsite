mod app;
mod components;
mod router;
mod static_articles;
mod switch;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
