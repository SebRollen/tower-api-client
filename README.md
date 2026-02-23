# tower-api-client

[![Crates.io][crates-badge]][crates-url]
[![Documentation][docs-badge]][docs-url]

[crates-badge]: https://img.shields.io/crates/v/tower-api-client.svg
[crates-url]: https://crates.io/crates/tower-api-client
[docs-badge]: https://img.shields.io/docsrs/tower-api-client
[docs-url]: https://docs.rs/tower-api-client

A lightweight Rust library for building strongly typed REST API clients, with built-in support for authentication, multiple request/response formats, pagination, and [Tower](https://github.com/tower-rs/tower) middleware integration.

## Features

- Strongly typed requests and responses via a `Request` trait
- Four authentication strategies: Bearer token, HTTP Basic, query parameter, custom headers
- Request data formats: JSON, form-encoded, query string, or empty
- Pagination support via a `PaginatedRequest` trait and async `Stream`
- Full Tower `Service` compatibility — compose with rate limiting, filtering, retries, and more

## Usage

### Defining a request

Implement the `Request` trait for your request type:

```rust
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use tower_api_client::{Request, RequestData};

#[derive(Serialize)]
struct GetUsers {
    page: usize,
}

#[derive(Deserialize)]
struct UsersResponse {
    data: Vec<String>,
}

impl Request for GetUsers {
    type Data = Self;
    type Response = UsersResponse;

    fn endpoint(&self) -> Cow<'_, str> {
        "/users".into()
    }

    fn data(&self) -> RequestData<&Self> {
        RequestData::Query(self)
    }
}
```

### Making a request

```rust
use tower::ServiceExt;
use tower_api_client::Client;

#[tokio::main]
async fn main() {
    let client = Client::new("https://api.example.com");
    let response = client.oneshot(GetUsers { page: 1 }).await.unwrap();
    println!("{:?}", response.data);
}
```

### Authentication

```rust
// Bearer token
let client = Client::new("https://api.example.com")
    .bearer_auth("my-token");

// HTTP Basic
let client = Client::new("https://api.example.com")
    .basic_auth("username", "password");

// Query parameter(s)
let client = Client::new("https://api.example.com")
    .query_auth(vec![("api_key", "my-key")]);

// Custom header(s)
let client = Client::new("https://api.example.com")
    .header_auth(vec![("X-Api-Key", "my-key")]);
```

### Tower middleware

`Client` implements Tower's `Service` trait, so it can be wrapped with any Tower middleware:

```rust
use std::time::Duration;
use tower::ServiceBuilder;
use tower_api_client::Client;

let client = ServiceBuilder::new()
    .rate_limit(10, Duration::from_secs(1))
    .service(Client::new("https://api.example.com"));
```

### Pagination

Implement `PaginatedRequest` and use `.paginate()` to get an async `Stream` of pages:

```rust
use tower_api_client::{pagination::PaginatedRequest, ServiceExt};

impl PaginatedRequest for GetUsers {
    type PaginationData = usize;

    fn get_page(&self) -> Option<usize> {
        Some(self.page)
    }

    fn next_page(&self, _prev: Option<&usize>, response: &UsersResponse) -> Option<usize> {
        // Return None to stop, or Some(next_page) to continue
        Some(self.page + 1)
    }

    fn update_request(&mut self, page: &usize) {
        self.page = *page;
    }
}

// Stream all pages
use futures::StreamExt;
let mut pages = client.paginate(GetUsers { page: 1 });
while let Some(result) = pages.next().await {
    println!("{:?}", result.unwrap().data);
}
```

## Request data formats

| Variant              | Description                          |
|----------------------|--------------------------------------|
| `RequestData::Empty` | No body or query string (default)    |
| `RequestData::Json`  | JSON body (`Content-Type: application/json`) |
| `RequestData::Form`  | URL-encoded form body                |
| `RequestData::Query` | Query string parameters              |

## License

MIT
