+++
title = "Controllers"
description = ""
date = 2021-05-01T18:10:00+00:00
updated = 2021-05-01T18:10:00+00:00
draft = false
weight = 5
sort_by = "weight"
template = "docs/page.html"

[extra]
lead = ""
toc = true
top = false
flair =[]
+++

`Loco` is a framework that wraps around [axum](https://crates.io/crates/axum), offering a straightforward approach to manage routes, middlewares, authentication, and more right out of the box. At any point, you can leverage the powerful axum Router and extend it with your custom middlewares and routes.

# Controllers and Routing

## Adding a controller

Provides a convenient code generator to simplify the creation of a starter controller connected to your project. Additionally, a test file is generated, enabling easy testing of your controller.

Generate a controller:

```sh
cargo loco generate controller [OPTIONS] <CONTROLLER_NAME>
```

After generating the controller, navigate to the created file in `src/controllers` to view the controller endpoints. You can also check the testing (in folder tests/requests) documentation for testing this controller.

### Displaying active routes

To view a list of all your registered controllers, execute the following command:

```sh
$ cargo loco routes

[GET] /_health
[GET] /_ping
[POST] /auth/forgot
[POST] /auth/login
[POST] /auth/register
[POST] /auth/reset
[POST] /auth/verify
[GET] /auth/current
```

This command will provide you with a comprehensive overview of the controllers currently registered in your system.

## AppRoutes

`AppRoutes` is a core component of the `Loco` framework that helps you manage and organize your application's routes. It provides a convenient way to add, prefix, and collect routes from different controllers.

### Features

- **Add Routes**: Easily add routes from different controllers.
- **Prefix Routes**: Apply a common prefix to a group of routes.
- **Collect Routes**: Gather all routes into a single collection for further processing.

### Examples

#### Adding Routes

You can add routes from different controllers to `AppRoutes`:

```rust
use loco_rs::controller::AppRoutes;
use loco_rs::prelude::*;
use axum::routing::get;

fn routes(_ctx: &AppContext) -> AppRoutes {
  AppRoutes::empty()
          .add_route(Routes::new().add("/", get(home_handler)))
          .add_route(Routes::new().add("/about", get(about_handler)))
}
```

### Prefixing Routes

Apply a common prefix to a group of routes:

```rust
use loco_rs::controller::AppRoutes;
use loco_rs::prelude::*;
use axum::routing::get;

fn routes(_ctx: &AppContext) -> AppRoutes {
    AppRoutes::empty()
        .prefix("/api")
        .add_route(Routes::new().add("/users", get(users_handler)))
        .add_route(Routes::new().add("/posts", get(posts_handler)))
}
```

### Nesting Routes

AppRoutes allows you to nest routes, making it easier to organize and manage complex route hierarchies.
This is particularly useful when you have a set of related routes that share a common prefix.

```rust
 use loco_rs::controller::AppRoutes;
use loco_rs::prelude::*;
use axum::routing::get;

fn routes(_ctx: &AppContext) -> AppRoutes {
  let route = Routes::new().add("/", get(|| async { "notes" }));
  AppRoutes::with_default_routes()
        .prefix("api")
        .add_route(controllers::auth::routes())
        .nest_prefix("v1")
        .nest_route("/notes", route)
}
```

## Adding state

Your app context and state is held in `AppContext` and is what Loco provides and sets up for you. There are cases where you'd want to load custom data,
logic, or entities when the app starts and be available to use in all controllers.

You could do that by using Axum's `Extension`. Here's an example for loading an LLM model, which is a time consuming task, and then providing it to a controller endpoint, where its already loaded, and fresh for use.

First, add a lifecycle hook in `src/app.rs`:

```rust
    // in src/app.rs, in your Hooks trait impl override the `after_routes` hook:

    async fn after_routes(router: axum::Router, _ctx: &AppContext) -> Result<axum::Router> {
        // cache should reside at: ~/.cache/huggingface/hub
        println!("loading model");
        let model = Llama::builder()
            .with_source(LlamaSource::llama_7b_code())
            .build()
            .unwrap();
        println!("model ready");
        let st = Arc::new(RwLock::new(model));

        Ok(router.layer(Extension(st)))
    }
```

Next, consume this state extension anywhere you like. Here's an example controller endpoint:

```rust
async fn candle_llm(Extension(m): Extension<Arc<RwLock<Llama>>>) -> impl IntoResponse {
    // use `m` from your state extension
    let prompt = "write binary search";
    ...
}
```

## Global app-wide state

Sometimes you might want state that can be shared between controllers, workers, and other areas of your app.

You can review the example [shared-global-state](https://github.com/loco-rs/shared-global-state) app to see how to integrate `libvips`, which is a C based image manipulation library. `libvips` requires an odd thing from the developer: to keep a single instance of it loaded per app process. We do this by keeping a [single `lazy_static` field](https://github.com/loco-rs/shared-global-state/blob/main/src/app.rs#L27-L34), and referring to it from different places in the app.

Read the following to see how it's done in each individual part of the app.

### Shared state in controllers

You can use the solution provided in this document. A live example [is here](https://github.com/loco-rs/loco/blob/master/examples/llm-candle-inference/src/app.rs#L41).

### Shared state in workers

Workers are intentionally verbatim initialized in [app hooks](https://github.com/loco-rs/loco/blob/master/starters/saas/src/app.rs#L59).

This means you can shape them as a "regular" Rust struct that takes a state as a field. Then refer to that field in perform.

[Here's how the worker is initialized](https://github.com/loco-rs/shared-global-state/blob/main/src/workers/downloader.rs#L19) with the global `vips` instance in the `shared-global-state` example.

Note that by-design _sharing state between controllers and workers have no meaning_, because even though you may choose to run workers in the same process as controllers initially (and share state) -- you'd want to quickly switch to proper workers backed by queue and running in a standalone workers process as you scale horizontally, and so workers should by-design have no shared state with controllers, for your own good.

### Shared state in tasks

Tasks don't really have a value for shared state, as they have a similar life as any exec'd binary. The process fires up, boots, creates all resources needed (connects to db, etc.), performs the task logic, and then the

## Routes in Controllers

Controllers define Loco routes capabilities. In the example below, a controller creates one GET endpoint and one POST endpoint:

```rust
use axum::routing::{get, post};
Routes::new()
    .add("/", get(hello))
    .add("/echo", post(echo))
```

You can also define a `prefix` for all routes in a controller using the `prefix` function.

## Sending Responses

Response senders are in the `format` module. Here are a few ways to send responses from your routes:

```rust

// keep a best practice of returning a `Result<impl IntoResponse>` to be able to swap return types transparently
pub async fn list(...) -> Result<impl IntoResponse> // ..

// use `json`, `html` or `text` for simple responses
format::json(item)


// use `render` for a builder interface for more involved responses. you can still terminate with
// `json`, `html`, or `text`
format::render()
    .etag("foobar")?
    .json(Entity::find().all(&ctx.db).await?)
```

### Content type aware responses

You can opt-in into the responders mechanism, where a format type is detected
and handed to you.

Use the `Format` extractor for this:

```rust
pub async fn get_one(
    respond_to: RespondTo,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    let res = load_item(&ctx, id).await?;
    match respond_to {
        RespondTo::Html => format::html(&format!("<html><body>{:?}</body></html>", item.title)),
        _ => format::json(item),
    }
}
```

### Custom errors

Here is a case where you might want to both render differently based on
different formats AND ALSO, render differently based on kinds of errors you got.

````rust
pub async fn get_one(
    respond_to: RespondTo,
    Path(id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Response> {
    // having `load_item` is useful because inside the function you can call and use
    // '?' to bubble up errors, then, in here, we centralize handling of errors.
    // if you want to freely use code statements with no wrapping function, you can
    // use the experimental `try` feature in Rust where you can do:
    // ```
    // let res = try {
    //     ...
    //     ...
    // }
    //
    // match res { ..}
    // ```
    let res = load_item(&ctx, id).await;

    match res {
        // we're good, let's render the item based on content type
        Ok(item) => match respond_to {
            RespondTo::Html => format::html(&format!("<html><body>{:?}</body></html>", item.title)),
            _ => format::json(item),
        },
        // we have an opinion how to render out validation errors, only in HTML content
        Err(Error::Model(ModelError::Validation(errors))) => match respond_to {
            RespondTo::Html => {
                format::html(&format!("<html><body>errors: {errors:?}</body></html>"))
            }
            _ => bad_request("opaque message: cannot respond!"),
        },
        // we have no clue what this is, let the framework render default errors
        Err(err) => Err(err),
    }
}
````

Here, we also "centralize" our error handling by first wrapping the workflow in a function, and grabbing the result type.

Next we create a 2 level match to:

1. Match the result type
2. Match the format type

Where we lack the knowledge for handling, we just return the error as-is and let the framework render out default errors.

## Creating a Controller Manually

#### 1. Create a Controller File

Start by creating a new file under the path `src/controllers`. For example, let's create a file named `example.rs`.

#### 2. Load the File in mod.rs

Ensure that you load the newly created controller file in the `mod.rs` file within the `src/controllers` folder.

#### 3. Register the Controller in App Hooks

In your App hook implementation (e.g., App struct), add your controller's `Routes` to `AppRoutes`:

```rust
// src/app.rs

pub struct App;
#[async_trait]
impl Hooks for App {
    fn routes() -> AppRoutes {
        AppRoutes::with_default_routes().prefix("prefix")
            .add_route(controllers::example::routes())
    }
    ...
}

```

# Middleware

Loco comes with a set of built-in middleware out of the box. Some are enabled by default, while others need to be configured. Middleware registration is flexible and can be managed either through the `*.yaml` environment configuration or directly in the code.

## The default stack

You get all the enabled middlewares run the following command

<!-- <snip id="cli-middleware-list" inject_from="yaml" template="sh"> -->

```sh
cargo loco middleware --config
```

<!-- </snip> -->

This is the stack in `development` mode:

```sh
$ cargo loco middleware --config

limit_payload          {"body_limit":{"Limit":1000000}}
cors                   {"enable":true,"allow_origins":["any"],"allow_headers":["*"],"allow_methods":["*"],"max_age":null,"vary":["origin","access-control-request-method","access-control-request-headers"]}
catch_panic            {"enable":true}
etag                   {"enable":true}
logger                 {"config":{"enable":true},"environment":"development"}
request_id             {"enable":true}
fallback               {"enable":true,"code":200,"file":null,"not_found":null}
powered_by             {"ident":"loco.rs"}


remote_ip              (disabled)
compression            (disabled)
timeout                (disabled)
static_assets          (disabled)
secure_headers         (disabled)
```

### Example: disable all middleware

Take what ever is enabled, and use `enable: false` with the relevant field. If `middlewares:` section in `server` is missing, add it.

```yaml
server:
  middlewares:
    cors:
      enable: false
    catch_panic:
      enable: false
    etag:
      enable: false
    logger:
      enable: false
    request_id:
      enable: false
    fallback:
      enable: false
```

The result:

```sh
$ cargo loco middleware --config
powered_by             {"ident":"loco.rs"}


cors                   (disabled)
catch_panic            (disabled)
etag                   (disabled)
remote_ip              (disabled)
compression            (disabled)
timeout_request        (disabled)
static                 (disabled)
secure_headers         (disabled)
logger                 (disabled)
request_id             (disabled)
fallback               (disabled)
```

You can control the `powered_by` middleware by changing the value for `server.ident`:

```yaml
server:
  ident: my-server #(or empty string to disable)
```

### Example: add a non-default middleware

Lets add the _Remote IP_ middleware to the stack. This is done just by configuration:

```yaml
server:
  middlewares:
    remote_ip:
      enable: true
```

The result:

```sh
$ cargo loco middleware --config

limit_payload          {"body_limit":{"Limit":1000000}}
cors                   {"enable":true,"allow_origins":["any"],"allow_headers":["*"],"allow_methods":["*"],"max_age":null,"vary":["origin","access-control-request-method","access-control-request-headers"]}
catch_panic            {"enable":true}
etag                   {"enable":true}
remote_ip              {"enable":true,"trusted_proxies":null}
logger                 {"config":{"enable":true},"environment":"development"}
request_id             {"enable":true}
fallback               {"enable":true,"code":200,"file":null,"not_found":null}
powered_by             {"ident":"loco.rs"}
```

### Example: change a configuration for an enabled middleware

Let's change the request body limit to `5mb`. When overriding a middleware configuration, rememeber to keep an `enable: true`:

```yaml
middlewares:
  limit_payload:
    body_limit: 5mb
```

The result:

```sh
$ cargo loco middleware --config

limit_payload          {"body_limit":{"Limit":5000000}}
cors                   {"enable":true,"allow_origins":["any"],"allow_headers":["*"],"allow_methods":["*"],"max_age":null,"vary":["origin","access-control-request-method","access-control-request-headers"]}
catch_panic            {"enable":true}
etag                   {"enable":true}
logger                 {"config":{"enable":true},"environment":"development"}
request_id             {"enable":true}
fallback               {"enable":true,"code":200,"file":null,"not_found":null}
powered_by             {"ident":"loco.rs"}


remote_ip              (disabled)
compression            (disabled)
timeout_request        (disabled)
static                 (disabled)
secure_headers         (disabled)
```

### Authentication

In the `Loco` framework, middleware plays a crucial role in authentication. `Loco` supports various authentication methods, including JSON Web Token (JWT) and API Key authentication. This section outlines how to configure and use authentication middleware in your application.

#### JSON Web Token (JWT)

##### Configuration

By default, Loco uses Bearer authentication for JWT. However, you can customize this behavior in the configuration file under the auth.jwt section.

- _Bearer Authentication:_ Keep the configuration blank or explicitly set it as follows:

  ```yaml
  # Authentication Configuration
  auth:
    # JWT authentication
    jwt:
      location:
        from: Bearer
  ```

- _Cookie Authentication:_ Configure the location from which to extract the token and specify the cookie name:

  ```yaml
  # Authentication Configuration
  auth:
    # JWT authentication
    jwt:
      location:
        from: Cookie
        name: token
  ```

- _Query Parameter Authentication:_ Specify the location and name of the query parameter:

  ```yaml
  # Authentication Configuration
  auth:
    # JWT authentication
    jwt:
      location:
        from: Query
        name: token
  ```

###### Multiple Location Authentication (Fallback Chain)

You can configure multiple authentication locations that will be tried in order until one succeeds. This is useful when you want to support multiple authentication methods simultaneously:

```yaml
# Authentication Configuration
auth:
  # JWT authentication
  jwt:
    location:
      - from: Bearer # Try Authorization header first
      - from: Cookie # If not found, try cookie
        name: session_token
      - from: Query # Finally try query parameter
        name: api_token
```

With this configuration:

1. The system first checks for a Bearer token in the `Authorization` header
2. If not found, it looks for a cookie named `session_token`
3. If still not found, it checks for a query parameter named `api_token`
4. If none are found, authentication fails with a descriptive error message

This approach provides flexibility for different client types:

- Web browsers can use cookies
- API clients can use Bearer tokens
- Simple clients can use query parameters

**Error Messages:** When authentication fails, the system provides clear feedback about which locations were attempted:

```
Token not found in any of the configured locations: [Bearer header, Cookie 'session_token', Query parameter 'api_token']
```

##### Usage

In your controller parameters, use `auth::JWT` for authentication. This triggers authentication validation based on the configured settings.

```rust
use loco_rs::prelude::*;

async fn current(
    auth: auth::JWT,
    State(_ctx): State<AppContext>,
) -> Result<Response> {
    // Your implementation here
}
```

Additionally, you can fetch the current user by replacing auth::JWT with `auth::ApiToken<users::Model>`.

#### API Key

For API Key authentication, use auth::ApiToken. This middleware validates the API key against the user database record and loads the corresponding user into the authentication parameter.

```rust
use loco_rs::prelude::*;

async fn current(
    auth: auth::ApiToken<users::Model>,
    State(_ctx): State<AppContext>,
) -> Result<Response> {
    // Your implementation here
}
```

## Catch Panic

This middleware catches panics that occur during request handling in the application. When a panic occurs, it logs the error and returns an internal server error response. This middleware helps ensure that the application can gracefully handle unexpected errors without crashing the server.

To disable the middleware edit the configuration as follows:

```yaml
#...
middlewares:
  catch_panic:
    enable: false
```

## Limit Payload

The Limit Payload middleware restricts the maximum allowed size for HTTP request payloads. By default, it is enabled and configured with a 2MB limit.

You can customize or disable this behavior through your configuration file.

### Set a custom limit

```yaml
#...
middlewares:
  limit_payload:
    body_limit: 5mb
```

### Disable payload size limitation

To remove the restriction entirely, set `body_limit` to `disable`:

```yaml
#...
middlewares:
  limit_payload:
    body_limit: disable
```

##### Usage

In your controller parameters, use `axum::body::Bytes`.

```rust
use loco_rs::prelude::*;

async fn current(_body: axum::body::Bytes,) -> Result<Response> {
    // Your implementation here
}
```

## Timeout

Applies a timeout to requests processed by the application. The middleware ensures that requests do not run beyond the specified timeout period, improving the overall performance and responsiveness of the application.

If a request exceeds the specified timeout duration, the middleware will return a `408 Request Timeout` status code to the client, indicating that the request took too long to process.

To enable the middleware edit the configuration as follows:

```yaml
#...
middlewares:
  timeout_request:
    enable: false
    timeout: 5000
```

## Logger

Provides logging functionality for HTTP requests. Detailed information about each request, such as the HTTP method, URI, version, user agent, and an associated request ID. Additionally, it integrates the application's runtime environment into the log context, allowing environment-specific logging (e.g., "development", "production").

To disable the middleware edit the configuration as follows:

```yaml
#...
middlewares:
  logger:
    enable: false
```

## Fallback

When choosing the SaaS starter (or any starter that is not API-first), you get a default fallback behavior with the _Loco welcome screen_. This is a development-only mode where a `404` request shows you a nice and friendly page that tells you what happened and what to do next. This also takes preference over the static handler, so make sure to disable it if you want to have static content served.

You can disable or customize this behavior in your `development.yaml` file. You can set a few options:

```yaml
# the default pre-baked welcome screen
fallback:
  enable: true
```

```yaml
# a different predefined 404 page
fallback:
  enable: true
  file: assets/404.html
```

```yaml
# a message, and customizing the status code to return 200 instead of 404
fallback:
  enable: true
  code: 200
  not_found: cannot find this resource
```

For production, it's recommended to disable this.

```yaml
# disable. you can also remove the `fallback` section entirely to disable
fallback:
  enable: false
```

## Remote IP

When your app is under a proxy or a load balancer (e.g. Nginx, ELB, etc.), it does not face the internet directly, which is why if you want to find out the connecting client IP, you'll get a socket which indicates an IP that is actually your load balancer instead.

The load balancer or proxy is responsible for doing the socket work against the real client IP, and then giving your app the load via the proxy back connection to your app.

This is why when your app has a concrete business need for getting the real client IP you need to use the de-facto standard proxies and load balancers use for handing you this information: the `X-Forwarded-For` header.

Loco provides the `remote_ip` section for configuring the `RemoteIP` middleware:

```yaml
server:
  middleware:
    # calculate remote IP based on `X-Forwarded-For` when behind a proxy or load balancer
    # use RemoteIP(..) extractor to get the remote IP.
    # without this middleware, you'll get the proxy IP instead.
    # For more: https://github.com/rails/rails/blob/main/actionpack/lib/action_dispatch/middleware/remote_ip.rb
    #
    # NOTE! only enable when under a proxy, otherwise this can lead to IP spoofing vulnerabilities
    # trust me, you'll know if you need this middleware.
    remote_ip:
      enable: true
      # # replace the default trusted proxies:
      # trusted_proxies:
      # - ip range 1
      # - ip range 2 ..
    # Generating a unique request ID and enhancing logging with additional information such as the start and completion of request processing, latency, status code, and other request details.
```

Then, use the `RemoteIP` extractor to get the IP:

```rust
#[debug_handler]
pub async fn list(ip: RemoteIP, State(ctx): State<AppContext>) -> Result<Response> {
    println!("remote ip {ip}");
    format::json(Entity::find().all(&ctx.db).await?)
}
```

When using the `RemoteIP` middleware, take note of the security implications vs. your current architecture (as noted in the documentation and in the configuration section): if your app is NOT under a proxy, you can be prone to IP spoofing vulnerability because anyone can set headers to arbitrary values, and specifically, anyone can set the `X-Forwarded-For` header.

This middleware is not enabled by default. Usually, you _will know_ if you need this middleware and you will be aware of the security aspects of using it in the correct architecture. If you're not sure -- don't use it (keep `enable` to `false`).

## Secure Headers

Loco comes with default secure headers applied by the `secure_headers` middleware. This is similar to what is done in the Rails ecosystem with [secure_headers](https://github.com/github/secure_headers).

In your `server.middleware` YAML section you will find the `github` preset by default (which is what Github and Twitter recommend for secure headers).

```yaml
server:
  middleware:
    # set secure headers
    secure_headers:
      preset: github
```

You can also override select headers:

```yaml
server:
  middleware:
    # set secure headers
    secure_headers:
      preset: github
      overrides:
        foo: bar
```

Or start from scratch:

```yaml
server:
  middleware:
    # set secure headers
    secure_headers:
      preset: empty
      overrides:
        foo: bar
```

To support `htmx`, You can add the following override, to allow some inline running of scripts:

```yaml
secure_headers:
  preset: github
  overrides:
    # this allows you to use HTMX, and has unsafe-inline. Remove or consider in production
    "Content-Security-Policy": "default-src 'self' https:; font-src 'self' https: data:; img-src 'self' https: data:; object-src 'none'; script-src 'unsafe-inline' 'self' https:; style-src 'self' https: 'unsafe-inline'"
```

## Compression

`Loco` leverages [CompressionLayer](https://docs.rs/tower-http/0.5.0/tower_http/compression/index.html) to enable a `one click` solution.

To enable response compression, based on `accept-encoding` request header, simply edit the configuration as follows:

```yaml
#...
middlewares:
  compression:
    enable: true
```

Doing so will compress each response and set `content-encoding` response header accordingly.

## Static Assets

The static assets middleware serves static files (e.g., images, CSS, JS) from a specified folder to the client. It also allows configuration of a fallback file to serve in case a requested file is not found, and can serve precompressed files if enabled.

### Basic Configuration

```yaml
#...
middlewares:
  static_assets:
    enable: true
    folder:
      uri: "/static"
      path: "assets/static"
    fallback: "assets/static/404.html"
    must_exist: true
```

### Cache Control

You can configure cache control headers for static assets to optimize performance. By default, static assets are cached for 1 year (`max-age=31536000`).

```yaml
#...
middlewares:
  static_assets:
    enable: true
    cache_control: "max-age=31536000, public"  # 1 year cache
    # or
    cache_control: "max-age=86400"  # 1 day cache
    # or
    cache_control: "no-cache"  # No caching
    # or
    cache_control: null  # Disable caching entirely
```

### Precompressed Assets

`Loco` leverages [ServeDir::precompressed_gzip](https://docs.rs/tower-http/latest/tower_http/services/struct.ServeDir.html#method.precompressed_gzip) to enable a `one click` solution of serving pre compressed assets.

If a static assets exists on the disk as a `.gz` file, `Loco` will serve it instead of compressing it on the fly.

```yaml
#...
middlewares:
  static_assets:
    enable: true
    precompressed: true
```

## CORS

This middleware enables Cross-Origin Resource Sharing (CORS) by allowing configurable origins, methods, and headers in HTTP requests.
It can be tailored to fit various application requirements, supporting permissive CORS or specific rules as defined in the middleware configuration.

```yaml
#...
middlewares:
  ...
  cors:
    enable: true
    # Set the value of the [`Access-Control-Allow-Origin`][mdn] header
    # allow_origins:
    #   - https://loco.rs
    # Set the value of the [`Access-Control-Allow-Headers`][mdn] header
    # allow_headers:
    # - Content-Type
    # Set the value of the [`Access-Control-Allow-Methods`][mdn] header
    # allow_methods:
    #   - POST
    # Set the value of the [`Access-Control-Max-Age`][mdn] header in seconds
    # max_age: 3600

```

## Handler and Route based middleware

`Loco` also allow us to apply [layers](https://docs.rs/tower/latest/tower/trait.Layer.html) to specific handlers or
routes.
For more information on handler and route based middleware, refer to the [middleware](/docs/the-app/controller/#middleware)
documentation.

### Handler based middleware

Apply a layer to a specific handler using `layer` method.

```rust
// src/controllers/auth.rs
pub fn routes() -> Routes {
    Routes::new()
        .prefix("auth")
        .add("/register", post(register).layer(middlewares::log::LogLayer::new()))
}
```

### Route based middleware

Apply a layer to a specific route using `layer` method.

```rust
// src/main.rs
pub struct App;

#[async_trait]
impl Hooks for App {
    fn routes(_ctx: &AppContext) -> AppRoutes {
        AppRoutes::with_default_routes()
            .add_route(
                controllers::auth::routes()
                    .layer(middlewares::log::LogLayer::new()),
            )
    }
}
```

# Request Validation

Request validation in the Loco framework ensures that incoming HTTP request data, such as JSON payloads, query parameters, or form data, conforms to predefined rules before processing. Utilizing the `validator` crate ([documentation](https://github.com/Keats/validator)), Loco provides specialized extractors for robust validation.

## Validation Extractors

| Extractor                    |  JSON  |  Content Type                     | Request Body                              |
|-------------------------------|------------------------|--------------------------------------------|-------------------------------------------|
| `JsonValidate`                   | ❌                     | `application/json`                         | JSON (e.g., `{"name": "John", "email": "john@example.com"}`) |
| `JsonValidateWithMessage`        | ✅                     | `application/json`                         | JSON (e.g., `{"name": "John", "email": "john@example.com"}`) |
| `QueryValidate`                   | ❌                     | Any                                        | Query string (e.g., `?name=John&email=john@example.com`) |
| `QueryValidateWithMessage`         | ✅                     | Any                                        | Query string (e.g., `?name=John&email=john@example.com`) |
| `FormValidate`                   | ❌                     | `application/x-www-form-urlencoded`         | Form data  |
| `FormValidateWithMessage`        | ✅                     | `application/x-www-form-urlencoded`         | Form data  |

### Notes:

- **Error Status**: HTTP status codes for invalid data or unsupported `Content-Type`.
- **Structured JSON Errors**: Provided by `WithMessage` extractors for detailed error reporting.
- **Supported Content Type**: Specifies the expected request `Content-Type`.
- **Request Body**: Describes the expected format of the request data.

## Implementing Request Validation

### 1. Define Validation Rules

Define a Rust struct with validation rules using the `serde` and `validator` crates.

#### Example: `DataParams` Struct

```rust
use serde::Deserialize;
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct DataParams {
    #[validate(length(min = 5, message = "Name must be at least 5 characters long"))]
    pub name: String,
    #[validate(email)]
    pub email: String,
}
```

- **Rules**:
  - `name`: Requires at least 5 characters.
  - `email`: Must be a valid email address.
- The `Validate` macro enables automatic field validation.

### 2. Create Handlers with Validation

Loco extractors validate data within HTTP handlers, proceeding with validated data or returning errors.

#### Example 1: JSON Validation with `JsonValidate`

```rust
use axum::debug_handler;
use loco_rs::prelude::*;

#[debug_handler]
pub async fn index(
    State(_ctx): State<AppContext>,
    JsonValidate(params): JsonValidate<DataParams>,
) -> Result<Response> {
    format::empty()
}
```

- Deserializes and validates JSON payloads (e.g., `{"name": "John", "email": "john@example.com"}`).
- Returns **400 Bad Request** on failure.

#### Example 2: Query Validation with `QueryValidate`

```rust
use axum::debug_handler;
use loco_rs::prelude::*;

#[debug_handler]
pub async fn index(
    State(_ctx): State<AppContext>,
    QueryValidate(params): QueryValidate<DataParams>,
) -> Result<Response> {
    format::empty()
}
```

- Validates query parameters (e.g., `?name=John&email=john@example.com`).
- Returns **400 Bad Request** on failure.

#### Example 3: Form Validation with `FormValidateWithMessage`

```rust
use axum::debug_handler;
use loco_rs::prelude::*;

#[debug_handler]
pub async fn index(
    State(_ctx): State<AppContext>,
    FormValidateWithMessage(params): FormValidateWithMessage<DataParams>,
) -> Result<Response> {
    format::empty()
}
```

- Validates form data (e.g., `name=John&email=john@example.com`) with `application/x-www-form-urlencoded` content type.
- Returns **400 Bad Request** on failure, with structured JSON errors.

### 3. Structured Error Responses

`WithMessage` extractors (e.g., `JsonValidateWithMessage`) provide detailed JSON error responses.

#### Example Error Response

For invalid form data (`{"name": "abc", "email": "invalid_email"}`):

```json
{
  "errors": {
    "name": [
      {
        "code": "length",
        "message": "Name must be at least 5 characters long",
        "params": {
          "min": 5,
          "value": "abc"
        }
      }
    ],
    "email": [
      {
        "code": "email",
        "message": null,
        "params": {
          "value": "invalid_email"
        }
      }
    ]
  }
}
```

- **Structure**:
  - `errors`: Maps fields to arrays of validation errors.
  - Each error includes `code`, `message`, and `params` for context.

## Key Considerations

- **Extractor Selection**: Choose based on the request data type (JSON, query, or form).
- **Error Handling**:
  - Standard extractors return generic HTTP errors.
  - `WithMessage` extractors provide structured JSON errors for client-side feedback.
- **Further Reading**: Refer to the [validator crate documentation](https://github.com/Keats/validator) for advanced validation rules.

# Pagination

In many scenarios, when querying data and returning responses to users, pagination is crucial. In `Loco`, we provide a straightforward method to paginate your data and maintain a consistent pagination response schema for your API responses.

We assume you have a `notes` entity and/or scaffold (replace this with any entity you like).

## Using pagination

```rust
use loco_rs::prelude::*;

let res = query::fetch_page(&ctx.db, notes::Entity::find(), &query::PaginationQuery::page(2)).await;
```

## Using pagination With Filter

```rust
use loco_rs::prelude::*;

let pagination_query = query::PaginationQuery {
    page_size: 100,
    page: 1,
};

let condition = query::condition().contains(notes::Column::Title, "loco");
let paginated_notes = query::paginate(
    &ctx.db,
    notes::Entity::find(),
    Some(condition.build()),
    &pagination_query,
)
.await?;
```

- Start by defining the entity you want to retrieve.
- Create your query condition (in this case, filtering rows that contain "loco" in the title column).
- Define the pagination parameters.
- Call the paginate function.

### Pagination view

After creating getting the `paginated_notes` in the previous example, you can choose which fields from the model you want to return and keep the same pagination response in all your different data responses.

Define the data you're returning to the user in Loco views. If you're not familiar with views, refer to the [documentation](@/docs/the-app/views.md) for more context.

Create a notes view file in `src/view/notes` with the following code:

```rust
use loco_rs::{
    controller::views::pagination::{Pager, PagerMeta},
    prelude::model::query::PaginatedResponse,
};
use serde::{Deserialize, Serialize};

use crate::models::_entities::notes;

#[derive(Debug, Deserialize, Serialize)]
pub struct ListResponse {
    id: i32,
    title: Option<String>,
    content: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PaginationResponse {}

impl From<notes::Model> for ListResponse {
    fn from(note: notes::Model) -> Self {
        Self {
            id: note.id.clone(),
            title: note.title.clone(),
            content: note.content,
        }
    }
}

impl PaginationResponse {
    #[must_use]
    pub fn response(data: PaginatedResponse<notes::Model>, pagination_query: &PaginationQuery) -> Pager<Vec<ListResponse>> {
        Pager {
            results: data
                .page
                .into_iter()
                .map(ListResponse::from)
                .collect::<Vec<ListResponse>>(),
            info: PagerMeta {
                page: pagination_query.page,
                page_size: pagination_query.page_size,
                total_pages: data.total_pages,
                total_items: data.total_items,
            },
        }
    }
}
```

# Testing

When testing controllers, the goal is to call the router's controller endpoint and verify the HTTP response, including the status code, response content, headers, and more.

To initialize a test request, use `use loco_rs::testing::prelude::*;`, which prepares your app routers, providing the request instance and the application context.

In the following example, we have a POST endpoint that returns the data sent in the POST request.

```rust
use loco_rs::testing::prelude::*;

#[tokio::test]
#[serial]
async fn can_print_echo() {
    configure_insta!();

    request::<App, _, _>(|request, _ctx| async move {
        let response = request
            .post("/example")
            .json(&serde_json::json!({"site": "Loco"}))
            .await;

        assert_debug_snapshot!((response.status_code(), response.text()));
    })
    .await;
}
```

As you can see initialize the testing request and using `request` instance calling /example endpoing.
the request returns a `Response` instance with the status code and the response test

## Async

When writing async tests with database data, it's important to ensure that one test does not affect the data used by other tests. Since async tests can run concurrently on the same database dataset, this can lead to unstable test results.

Instead of using `request`, as described in the documentation for synchronous tests, use the `request_with_create_db` function. This function generates a random database schema name and ensures that the tables are deleted once the test is completed.

Note: If you cancel the test run midway (e.g., by pressing `Ctrl + C`), the cleanup process will not execute, and the database tables will remain. In such cases, you will need to manually remove them.

```rust
use loco_rs::testing::prelude::*;

#[tokio::test]
async fn can_print_echo() {
    configure_insta!();

    request_with_create_db::<App, _, _>(|request, _ctx| async move {
        let response = request
            .post("/example")
            .json(&serde_json::json!({"site": "Loco"}))
            .await;

        assert_debug_snapshot!((response.status_code(), response.text()));
    })
    .await;
}
```

## Authenticated Endpoints

The following example works for both JWT and API_KEY Authentication.

```rust
use loco_rs::testing::prelude::*;
use super::prepare_data;

#[tokio::test]
#[serial]
async fn can_get_current_user() {
    configure_insta!();

    request::<App, _, _>(|request, ctx| async move {
        // Initialize the user
        let user = prepare_data::init_user_login(&request, &ctx).await;
        let (auth_key, auth_value) = prepare_data::auth_header(&user.token);

        // Then add the key to the request, usually in the header
        let response = request
            .get("/example")
            .add_header(auth_key, auth_value)
            .await;

        assert_eq!(
            response.status_code(),
            200,
            "Current request should succeed"
        );

        assert_debug_snapshot!((response.status_code(), response.text()));
    })
    .await;
}
```
