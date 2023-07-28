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
        Route::Login => html! { <Login/> },
        Route::AdminDashboard => {
            html! {
                <>
                    <Header/>
                    <WithSession>
                        <admin::Dashboard/>
                    </WithSession>
                </>
            }
        }
        Route::PasswordChange => {
            html! {<WithSession><admin::PasswordChange/></WithSession>}
        }
        Route::CreateArticle => {
            html! {<WithSession><ArticleEditor mode={ ArticleEditorMode::Create }/></WithSession>}
        }
        Route::EditArticle { public_id } => {
            html! {<WithSession><EditArticle {public_id}/></WithSession>}
        }
        Route::MarkdownPreview => {
            html! {<MarkdownPreviewPage/>}
        }
        Route::ArticleList => {
            html! {
                <>
                    <Header/>
                    <WithSession optional={true}>
                        <ArticleList/>
                    </WithSession>
                </>
            }
        }
        Route::ArticleViewer { public_id } => {
            match public_id.as_str() {
                _ if public_id == static_articles().md_article_editor.public_id => {
                    html! {
                        <DefaultStyling>
                            <MarkdownPreviewPage md={ include_str!("../md/md_post.md") } />
                        </DefaultStyling>
                    }
                }
                _ if public_id == static_articles().about.public_id => {
                    // TODO this place looks similar to ArticleViewer's
                    html! {
                        <DefaultStyling>
                            <Header/>
                            <PageTitle title={static_articles().about.title}/>
                            <Post md={include_str!("../../README.md")}/>
                        </DefaultStyling>
                    }
                }
                _ if public_id == static_articles().place.public_id => {
                    html! {
                        <DefaultStyling>
                            <Header/>
                            <Place/>
                        </DefaultStyling>
                    }
                }
                _ => html! {
                    <>
                        <Header/>
                        <ArticleViewer {public_id}/>
                    </>
                },
            }
        }
    }
}
