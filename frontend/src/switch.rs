use crate::router::Route;

use yew::prelude::*;

pub fn switch(routes: Route) -> Html {
    use crate::components::admin::WithSession;
    use crate::components::*;

    match routes {
        Route::NotFound => html! {<Colored with="red"><h1>{"not found 404"}</h1></Colored> },
        Route::Unauthorized => html! {<Colored with="red"><h1>{"unauthorized 401"}</h1></Colored> },
        Route::Home => html! {
            <WithTheme>
                <Post md={""}/>
            </WithTheme>
        },
        Route::Login => html! { <WithTheme><Login/></WithTheme> },
        Route::AdminDashboard => {
            html! {<WithSession><admin::Dashboard/></WithSession>}
        }
        Route::PasswordChange => {
            html! {<WithSession><admin::PasswordChange/></WithSession>}
        }
        Route::ArticleEditor => {
            html! {<WithTheme><WithSession><ArticleEditor mode={ ArticleEditorMode::Create }/></WithSession></WithTheme>}
        }
        Route::EditArticle { public_id } => {
            html! {<WithTheme><WithSession><EditArticle {public_id}/></WithSession></WithTheme>}
        }
        Route::MarkdownPreview => {
            html! {<WithTheme><MarkdownPreviewPage/></WithTheme>}
        }
        Route::ArticleList => {
            html! {<WithSession optional={true}><WithTheme><ArticleList/></WithTheme></WithSession>}
        }
        Route::ArticleViewer { public_id } => {
            html! {<WithTheme><ArticleViewer {public_id}/></WithTheme>}
        }
    }
}
