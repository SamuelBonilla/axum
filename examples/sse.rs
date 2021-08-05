//! Run with
//!
//! ```not_rust
//! cargo run --example sse --features=headers
//! ```

use axum::{extract::TypedHeader, prelude::*, routing::nest, service::ServiceExt, sse::Event};
use futures::stream::{self, Stream};
use http::StatusCode;
use std::{convert::Infallible, net::SocketAddr, time::Duration};
use tokio_stream::StreamExt as _;
use tower_http::{services::ServeDir, trace::TraceLayer};

#[tokio::main]
async fn main() {
    // Set the RUST_LOG, if it hasn't been explicitly defined
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var("RUST_LOG", "sse=debug,tower_http=debug")
    }
    tracing_subscriber::fmt::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    // build our application with a route
    let app = nest(
        "/",
        axum::service::get(
            ServeDir::new("examples/sse")
                .append_index_html_on_directories(true)
                .handle_error(|error: std::io::Error| {
                    Ok::<_, std::convert::Infallible>((
                        StatusCode::INTERNAL_SERVER_ERROR,
                        format!("Unhandled internal error: {}", error),
                    ))
                }),
        ),
    )
    .route("/sse", axum::sse::sse(make_stream))
    .layer(TraceLayer::new_for_http());

    // run it
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn make_stream(
    // sse handlers can also use extractors
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Result<impl Stream<Item = Result<Event, Infallible>>, Infallible> {
    println!("`{}` connected", user_agent.as_str());

    // A `Stream` that repeats an event every second
    let stream = stream::repeat_with(|| Event::default().data("hi!"))
        .map(Ok)
        .throttle(Duration::from_secs(1));

    Ok(stream)
}