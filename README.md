Source code behind <a href="https://phantie.com">phantie.com</a>.  

Fully supports articles with a strict schema stored in a database and with a liberal schema carved in source code, and an administration panel; partially implemented email subscription.

Its entirety is written in Rust: backend api, frontend and database. Core technologies for each part: backend api - Axum, frontend - Yew (Rust compiled to WebAssebly), database - Bonsaidb.

System can be horizontally scaled to the extent - backend api and frontend are stateless.

Interesting implemented things:
- frontend in Rust
- database self hosted with custom tooling and daily data auto backups using DigitalOcean Volumes
- found and reported or fixed several noteworthy bugs of BonsaiDB
- frontend supports any number of themes (I wanted to try myself in this long ago; switch the theme by pressing on a circle in the top right corner)
- compile-time routes to backend api methods
- database client tooling for its connection reliability
- custom admin session persistent storage layer (only in memory is provided by default)
- hosting on DigitalOcean Droplets
