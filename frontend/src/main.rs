mod app;
mod components;
mod router;
mod switch;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
