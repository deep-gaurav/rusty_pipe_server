use serde::{Serialize,Deserialize};

use async_trait::async_trait;
use rusty_pipe::downloader_trait::Downloader;
use rusty_pipe::youtube_extractor::stream_extractor::YTStreamExtractor;

use juniper::{EmptyMutation, EmptySubscription, FieldError, RootNode};
use warp::{http::Response, Filter};
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::str::FromStr;

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

    async fn download_with_header(&self, url: &str, header: HashMap<String, String, RandomState>) -> Result<String, String> {
        let client=reqwest::Client::new();
        let res = client.get(url);
        let mut headers = reqwest::header::HeaderMap::new();
        for header in header{
            headers.insert(reqwest::header::HeaderName::from_str(&header.0).map_err(|e|e.to_string())?, header.1.parse().unwrap());
        }
        let res = res.headers(headers);
        let res = res.send().await.map_err(|er|er.to_string())?;
        let body = res.text().await.map_err(|er|er.to_string())?;
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
    fn video_streams(&self) -> Result<Vec<StreamItem>, FieldError> {
        let streams = self.extractor.get_video_stream()?;
        let mut v = vec![];
        for stream in streams {
            let stream_str = serde_json::to_string(&stream)?;
            v.push(
                serde_json::from_str(&stream_str)?
            );
        }
        Ok(v)
    }
    fn video_only_streams(&self) -> Result<Vec<StreamItem>, FieldError> {
        let streams = self.extractor.get_video_only_stream()?;
        let mut v = vec![];
        for stream in streams {
            let stream_str = serde_json::to_string(&stream)?;
            v.push(
                serde_json::from_str(&stream_str)?
            );
        }
        Ok(v)
    }
    fn audio_only_streams(&self) -> Result<Vec<StreamItem>, FieldError> {
        let streams = self.extractor.get_audio_streams()?;
        let mut v = vec![];
        for stream in streams {
            let stream_str = serde_json::to_string(&stream)?;
            v.push(
                serde_json::from_str(&stream_str)?
            );
        }
        Ok(v)
    }

    fn title(&self)->Result<String,FieldError>{
        Ok(self.extractor.get_name()?)
    }

    fn description(&self)->Result<String,FieldError>{
        Ok(self.extractor.get_description(false)?.0)
    }

    fn uploader_name(&self)->Result<String,FieldError>{
        Ok(self.extractor.get_uploader_name()?)
    }

    fn uploader_url(&self)->Result<String,FieldError>{
        Ok(self.extractor.get_uploader_url()?)
    }

    fn video_thumbnails(&self)->Result<Vec<Thumbnail>,FieldError>{
        let thumbs = self.extractor.get_video_thumbnails()?;
        let mut thumbf = vec![];
        for thumb in thumbs{
            thumbf.push(
                Thumbnail{
                    url:thumb.url,
                    height: thumb.height as i32,
                    width: thumb.width as i32
                }
            )
        }
        Ok(thumbf)
    }

    fn uploader_thumbnails(&self)->Result<Vec<Thumbnail>,FieldError>{
        let thumbs = self.extractor.get_uploader_avatar_url()?;
        let mut thumbf = vec![];
        for thumb in thumbs{
            thumbf.push(
                Thumbnail{
                    url:thumb.url,
                    height: thumb.height as i32,
                    width: thumb.width as i32
                }
            )
        }
        Ok(thumbf)
    }

    fn likes(&self)->Result<i32,FieldError>{
        Ok(self.extractor.get_like_count()? as i32)
    }

    fn dislikes(&self)->Result<i32,FieldError>{
        Ok(self.extractor.get_dislike_count()? as i32)
    }

    fn views(&self)->Result<i32,FieldError>{
        Ok(self.extractor.get_view_count()? as i32)
    }

    fn length(&self)->Result<i32,FieldError>{
        Ok(self.extractor.get_length()? as i32)
    }

}

#[derive(juniper::GraphQLObject,Serialize,Deserialize)]
pub struct StreamItem {
    pub url: String,
    pub itag: i32,
    pub approxDurationMs: String,
    pub audioChannels: Option<i32>,
    pub audioQuality: Option<String>,
    pub audioSampleRate: Option<String>,
    pub averageBitrate: Option<i32>,
    pub bitrate: i32,
    pub contentLength: Option<String>,
    pub height: Option<i32>,
    pub width: Option<i32>,
    pub quality: String,
    pub qualityLabel: Option<String>,
    pub lastModified: String,
    pub mimeType: String,
}

#[derive(juniper::GraphQLObject,Serialize,Deserialize)]
pub struct Thumbnail{
    url:String,
    width:i32,
    height:i32
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
