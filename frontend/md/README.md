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

- frontend
    - support for any number of themes (button in the right top corner)
    - snake game
        - from scratch implementation with rendering using canvas
- backend
    - user online by keeping of open websocket connections
    - self hosted database with daily data auto backups using DigitalOcean Volumes
    - custom user session persistent storage layer
    - compile-time routes to backend api methods
    - found and reported or fixed several noteworthy bugs of BonsaiDB
