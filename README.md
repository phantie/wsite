Source code on <a href="https://github.com/phantie/wsite">Github</a>

[![CI-Tests](https://github.com/phantie/wsite/actions/workflows/testing.yml/badge.svg)](https://github.com/phantie/wsite/actions/workflows/testing.yml)

Supports articles CRUD and other features on admin panel.

Its entirety is written in Rust: backend api, frontend and database. Core technologies for each part: backend api - Axum, frontend - Yew (Rust compiled to WebAssebly), database - CozoDB, previously BonsaiDB.

Backend api and frontend are stateless, database only one instance.

Architecture
---------------
<!-- accessed from github, the second link should fail due to 404. accessed from deployment, the first should fail due to CORB -->
![](https://github.com/phantie/wsite/blob/master/backend/static/app-system-diagram.png)
![](/api/static/app-system-diagram.png)


Interesting implemented things:
--------------------------------------
- frontend in Rust
- database self hosted with custom tooling and daily data auto backups using DigitalOcean Volumes
- user online by keeping of open websocket connections
- frontend support for any number of themes (button in the right top corner)
- compile-time routes to backend api methods
- database client tooling for its connection reliability
- custom admin session persistent storage layer (only in memory is provided by default)
- hosting on DigitalOcean Droplets
- found and reported or fixed several noteworthy bugs of BonsaiDB
