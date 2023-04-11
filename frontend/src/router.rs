use yew_router::prelude::*;

// Router accepts only literals, so static_routes are used in tests
#[derive(Clone, Routable, PartialEq)]
pub enum Route {
    #[at("/")]
    Home,
    #[at("/login")]
    Login,
    #[at("/admin/dashboard")]
    AdminDashboard,
    #[not_found]
    #[at("/404")]
    NotFound,
    #[at("/401")]
    Unauthorized,
}

#[cfg(test)]
mod tests {
    use static_routes::*;
    use yew_router::Routable;

    use super::Route;

    fn map_to_one_another(frontend_defined_route: impl Routable, static_route: impl Get) {
        assert_eq!(
            frontend_defined_route.to_path(),
            static_route.get().complete()
        );
    }

    #[test]
    fn test_local_routes_map_to_static_routes() {
        let routes = routes().root;

        map_to_one_another(Route::Home, routes.home);
        map_to_one_another(Route::Login, routes.login);
        map_to_one_another(Route::AdminDashboard, routes.admin.dashboard);
    }
}
