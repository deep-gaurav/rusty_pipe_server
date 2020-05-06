//!
//! This example demonstrates async/await usage with warp.
//! NOTE: this uses tokio 0.1 , not the alpha tokio 0.2.

use async_trait::async_trait;
use rusty_pipe::downloader_trait::Downloader;
use rusty_pipe::youtube_extractor::stream_extractor::YTStreamExtractor;

use juniper::{EmptyMutation, EmptySubscription, FieldError, RootNode};
use warp::{http::Response, Filter};
// use juniper_warp::
struct DownloaderObj;

#[async_trait]
impl Downloader for DownloaderObj {
    async fn download(&self, url: &str) -> Result<String, String> {
        // println!("query url : {}",url);
        let resp = reqwest::get(url).await.map_err(|er| er.to_string())?;
        // println!("got response ");
        let body = resp.text().await.map_err(|er| er.to_string())?;
        // println!("suceess query");
        Ok(String::from(body))
    }
}

#[derive(Clone)]
struct Context {}
impl juniper::Context for Context {}

struct YoutubeStreamItem {
    extractor: YTStreamExtractor<DownloaderObj>,
}
#[juniper::graphql_object(Context = Context)]
impl YoutubeStreamItem {
    async fn video_streams(&self) -> Result<Vec<StreamItem>, FieldError> {
        let streams = self.extractor.get_video_stream().await?;
        let mut v = vec![];
        for stream in streams {
            v.push(StreamItem {
                url: stream.url,
                resolution: stream.itag.resolution_string,
            });
        }
        Ok(v)
    }
    async fn video_only_streams(&self) -> Result<Vec<StreamItem>, FieldError> {
        let streams = self.extractor.get_video_only_stream().await?;
        let mut v = vec![];
        for stream in streams {
            v.push(StreamItem {
                url: stream.url,
                resolution: stream.itag.resolution_string,
            });
        }
        Ok(v)
    }
    async fn audio_only_streams(&self) -> Result<Vec<StreamItem>, FieldError> {
        let streams = self.extractor.get_audio_streams().await?;
        let mut v = vec![];
        for stream in streams {
            v.push(StreamItem {
                url: stream.url,
                resolution: stream.itag.resolution_string,
            });
        }
        Ok(v)
    }
}

#[derive(juniper::GraphQLObject)]
struct StreamItem {
    url: String,
    resolution: String,
}

struct Query;

#[juniper::graphql_object(Context = Context)]
impl Query {
    async fn youtube_stream(video_id: String) -> Result<YoutubeStreamItem, FieldError> {
        let ytextractor = YTStreamExtractor::new(&video_id, DownloaderObj).await?;
        Ok(YoutubeStreamItem {
            extractor: ytextractor,
        })
    }
}

type Schema = RootNode<'static, Query, EmptyMutation<Context>, EmptySubscription<Context>>;

fn schema() -> Schema {
    Schema::new(
        Query,
        EmptyMutation::<Context>::new(),
        EmptySubscription::<Context>::new(),
    )
}

#[tokio::main]
async fn main() {
    ::std::env::set_var("RUST_LOG", "warp_async");
    env_logger::init();

    let log = warp::log("warp_server");

    let homepage = warp::path::end().map(|| {
        Response::builder()
            .header("content-type", "text/html")
            .body(format!(
                "<html><h1>juniper_warp</h1><div>visit <a href=\"/graphiql\">/graphiql</a></html>"
            ))
    });

    log::info!("Listening on 127.0.0.1:8080");

    let state = warp::any().map(move || Context {});
    let graphql_filter = juniper_warp::make_graphql_filter(schema(), state.boxed());

    warp::serve(
        warp::get()
            .and(warp::path("graphiql"))
            .and(juniper_warp::graphiql_filter("/graphql", None))
            .or(homepage)
            .or(warp::path("graphql").and(graphql_filter))
            .with(log),
    )
    .run((
        [0, 0, 0, 0],
        std::env::var("PORT")
            .unwrap_or("8080".to_owned())
            .parse()
            .unwrap(),
    ))
    .await
}
