Source code on <a href="https://github.com/phantie/wsite">Github</a>

[![CI-Tests](https://github.com/phantie/wsite/actions/workflows/testing.yml/badge.svg)](https://github.com/phantie/wsite/actions/workflows/testing.yml)

Core technologies: Axum, Yew, CozoDB.

Interesting implemented things:
--------------------------------------

- frontend
    - markdown parsing and custom rendering
    - support for colored code blocks
    - support for any number of themes (button in the right top corner)
    - snake game
        - from scratch implementation with rendering using canvas
- backend
    - user online by keeping of open websocket connections
    - usage of Datalog based database CozoDB for persistence
    - custom user session persistent storage layer
    - self hosted database with daily data auto backups using DigitalOcean Volumes
    - compile-time routes to backend api methods
    - found and reported or fixed several noteworthy bugs of BonsaiDB


Architecture
---------------
<!-- accessed from github, the second link should fail due to 404. accessed from deployment, the first should fail due to CORB -->
![](https://github.com/phantie/wsite/blob/master/backend/static/app-system-diagram.png)
![](/api/static/app-system-diagram.png)


![](https://phantie.site/api/endpoint_hits/github/wsite)