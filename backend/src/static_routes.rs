pub mod extend {
    use axum::response::Redirect;
    use static_routes::*;

    pub trait GetExtend {
        fn redirect_to(&self) -> Redirect;
    }

    impl<G: Get> GetExtend for G {
        fn redirect_to(&self) -> Redirect {
            Redirect::to(self.get().complete())
        }
    }
}
