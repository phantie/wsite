use crate::router::Route;

use crate::static_articles::static_articles;
use yew::prelude::*;

pub fn switch(routes: Route) -> Html {
    use crate::components::*;
    use admin::WithSession;

    let path = yew_router::Routable::to_path(&routes);
    let status: u16 = match routes.clone() {
        Route::NotFound => 404,
        Route::Unauthorized => 401,
        _ => 200,
    };

    let article_list = html! {
        <>
            <Header/>
            <WithSession optional={true}>
                <ArticleList/>
            </WithSession>
        </>
    };

    {
        use crate::components::imports::*;
        let params = web_sys::window().unwrap().location().search().unwrap();
        // TODO makes duplicate requests
        wasm_bindgen_futures::spawn_local(async move {
            let req = Request::static_post(routes().api.endpoint_hits.frontend)
                .json(&interfacing::FrontendEndpointHit {
                    endpoint: format!("{}{}", path.clone(), params),
                    status,
                })
                .unwrap()
                .send()
                .await;
            if let Err(_e) = req {
                gloo_console::log!(format!("failed request_register_endpoint_hit for {path:?}"))
            }
        });
    }

    #[allow(unused)]
    enum Home {
        ArticleList,
        Snake,
    }

    const HOME: Home = Home::Snake;

    use snake::comp::{NotBegunState, State};

    match routes {
        Route::Home => match HOME {
            Home::ArticleList => article_list.clone(),
            Home::Snake => html! { <Snake/> },
        },
        Route::Snake => {
            html! {
                <Snake/>
            }
        }
        Route::SnakeCreateJoinLobby => {
            let state = State::NotBegun {
                inner: NotBegunState::MPCreateJoinLobby,
            };
            html! {
                <Snake {state}/>
            }
        }
        Route::SnakeLobbies => {
            let state = State::NotBegun {
                inner: NotBegunState::MPSetUsername {
                    next_state: Box::new(NotBegunState::MPCreateJoinLobby),
                },
            };
            html! {
                <Snake {state}/>
            }
        }
        Route::SnakeCreateLobby => {
            let state = State::NotBegun {
                inner: NotBegunState::MPCreateLobby,
            };
            html! {
                <Snake {state}/>
            }
        }
        // TODO requires user_name setting
        Route::SnakeLobby { lobby_name } => {
            html! {
                // TODO refactor
                <Snake state={snake::comp::State::to_be_loaded_lobby(lobby_name) }/>
            }
        }
        Route::Ref => {
            html! { <yew_router::prelude::Redirect<Route> to={Route::Home}/> }
        }
        Route::ArticleList => article_list.clone(),
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
                            <Post md={include_str!("../md/README.md")}/>
                        </DefaultStyling>
                    }
                }
                _ if public_id == static_articles().snake.public_id => {
                    html! {
                        <Snake/>
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
        Route::NotFound => html! { <Error msg={"Not Found"} code=404 /> },
        Route::Unauthorized => html! { <Error msg={"Unauthorized"} code=401 /> },
    }
}
