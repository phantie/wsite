use crate::router::Route;

use crate::static_articles::static_articles;
use yew::prelude::*;

pub fn switch(routes: Route) -> Html {
    use crate::components::*;
    use admin::WithSession;

    match routes {
        Route::NotFound => html! {<Colored with="red"><h1>{"not found 404"}</h1></Colored> },
        Route::Unauthorized => html! {<Colored with="red"><h1>{"unauthorized 401"}</h1></Colored> },
        Route::Home => {
            html! { <yew_router::prelude::Redirect<Route> to={Route::ArticleList}/> }
        }
        Route::Login => html! { <WithTheme><Login/></WithTheme> },
        Route::AdminDashboard => {
            html! {<WithTheme><WithSession><admin::Dashboard/></WithSession></WithTheme>}
        }
        Route::PasswordChange => {
            html! {<WithTheme><WithSession><admin::PasswordChange/></WithSession></WithTheme>}
        }
        Route::CreateArticle => {
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
        Route::ArticleViewer { public_id } => match public_id.as_str() {
            _ if public_id == static_articles().md_article_editor.public_id => {
                html! {
                    <WithTheme>
                        <MarkdownPreviewPage md={ include_str!("../md/md_post.md") } />
                    </WithTheme>
                }
            }
            _ if public_id == static_articles().about.public_id => {
                // TODO this place looks similar to ArticleViewer's
                html! {
                    <WithTheme>
                        <DefaultStyling>
                            <PageTitle title={static_articles().about.title}/>
                            <Post md={include_str!("../../README.md")}/>
                        </DefaultStyling>
                    </WithTheme>
                }
            }
            _ if public_id == static_articles().place.public_id => {
                html! {
                    <WithTheme>
                        <DefaultStyling>
                            <Place/>
                        </DefaultStyling>
                    </WithTheme>
                }
            }
            _ => html! {<WithTheme><ArticleViewer {public_id}/></WithTheme>},
        },
    }
}
