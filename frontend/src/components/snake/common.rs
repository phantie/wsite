#[derive(Clone, Copy)]
pub struct WindowSize {
    pub width: i32,
    pub height: i32,
}

impl From<web_sys::Window> for WindowSize {
    fn from(value: web_sys::Window) -> Self {
        let width = value.inner_width().unwrap().as_f64().unwrap() as i32;
        let height = value.inner_height().unwrap().as_f64().unwrap() as i32;

        let width = width - 15;
        let height = height - 5;

        Self { width, height }
    }
}
