use axum::response::Html;

pub async fn login_form() -> Html<&'static str> {
    let html: &'static str = r#"
    <!DOCTYPE html>
    <html lang="en">
    
    <head>
        <meta http-equiv="content-type" content="text/html; charset=utf-8">
        <title>Login</title>
    </head>
    
    <body>
        <form action="/api/login" method="post">
            <label>Username
                <input type="text" placeholder="Enter Username" name="username">
            </label>
            <label>Password
                <input type="password" placeholder="Enter Password" name="password">
            </label>
            <button type="submit">Login</button>
        </form>
    </body>
    
    </html>
    "#;

    Html(html)
}
