## Blunt-CMS
Backend for Blunt-CMS using [actix-web](https://actix.rs/) framework in [rust](https://www.rust-lang.org/).

According to [Wikipedia](https://en.wikipedia.org/wiki/Content_management_system)-
*A content management system (CMS) is computer software used to manage the creation and modification of digital content (content management). A CMS is typically used for enterprise content management (ECM) and web content management (WCM).*

## Getting Started
1. Setup `rust` and `cargo` using official installation page [here](https://www.rust-lang.org/tools/install).
2. I am using [PostgreSQL](https://www.postgresql.org/) for database
3. Run `cargo run`
```console
cargo run
```
4. You can change configuration of the server like database url, etc. by changing [.env](.env) file.
5. Visit `http://{HOST}:{PORT}/api-doc/ui.html` for swagger-ui

## Copyrights
Licensed under [@MIT](./LICENSE)
