mod channel;
mod playlist;
mod search;
mod trending;

use serde::{Deserialize, Serialize};

use async_trait::async_trait;
use rusty_pipe::downloader_trait::Downloader;
use rusty_pipe::youtube_extractor::search_extractor::{YTSearchExtractor, YTSearchItem};
use rusty_pipe::youtube_extractor::stream_extractor::YTStreamExtractor;

use juniper::{EmptyMutation, EmptySubscription, FieldError, RootNode};
use rusty_pipe::youtube_extractor::error::ParsingError;
use search::Search;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::str::FromStr;
use warp::{http::Response, Filter};

use crate::channel::Channel;
use crate::playlist::Playlist;
use crate::trending::Trending;
use lazy_static::lazy_static;
use rusty_pipe::utils::utils::fix_thumbnail_url;
use rusty_pipe::youtube_extractor::channel_extractor::YTChannelExtractor;
use rusty_pipe::youtube_extractor::playlist_extractor::YTPlaylistExtractor;
use rusty_pipe::youtube_extractor::trending_extractor::YTTrendingExtractor;

use crate::search::*;

lazy_static! {
    static ref download_reqwest_client: reqwest::Client = reqwest::Client::new();
}

// use juniper_warp::
struct DownloaderObj;

#[async_trait]
impl Downloader for DownloaderObj {
    async fn download(url: &str) -> Result<String, ParsingError> {
        println!("query url : {}", url);
        let resp = reqwest::get(url)
            .await
            .map_err(|er| ParsingError::DownloadError {
                cause: er.to_string(),
            })?;
        println!("got response ");
        let body = resp
            .text()
            .await
            .map_err(|er| ParsingError::DownloadError {
                cause: er.to_string(),
            })?;
        println!("suceess query");
        Ok(String::from(body))
    }

    async fn download_with_header(
        url: &str,
        header: HashMap<String, String>,
    ) -> Result<String, ParsingError> {
        let client = reqwest::Client::new();
        let res = client.get(url);
        let mut headers = reqwest::header::HeaderMap::new();
        for header in header {
            headers.insert(
                reqwest::header::HeaderName::from_str(&header.0).map_err(|e| e.to_string())?,
                header.1.parse().unwrap(),
            );
        }
        let res = res.headers(headers);
        let res = res.send().await.map_err(|er| er.to_string())?;
        let body = res.text().await.map_err(|er| er.to_string())?;
        Ok(String::from(body))
    }

    fn eval_js(script: &str) -> Result<String, String> {
        use quick_js::{Context, JsValue};
        let context = Context::new().expect("Cant create js context");
        // println!("decryption code \n{}",decryption_code);
        // println!("signature : {}",encrypted_sig);
        println!("jscode \n{}", script);
        let res = context.eval(script).unwrap_or(quick_js::JsValue::Null);
        // println!("js result : {:?}", result);
        let result = res.into_string().unwrap_or("".to_string());
        print!("JS result: {}", result);
        Ok(result)
    }
}

#[derive(Clone)]
pub struct Context {}
impl juniper::Context for Context {}

struct Video {
    extractor: YTStreamExtractor<DownloaderObj>,
}

#[juniper::graphql_object(Context = Context)]
impl Video {
    fn video_streams(&self) -> Result<Vec<StreamItem>, FieldError> {
        let streams = self.extractor.get_video_stream()?;
        let mut v = vec![];
        for stream in streams {
            let stream_str = serde_json::to_string(&stream)?;
            v.push(serde_json::from_str(&stream_str)?);
        }
        Ok(v)
    }
    fn video_only_streams(&self) -> Result<Vec<StreamItem>, FieldError> {
        let streams = self.extractor.get_video_only_stream()?;
        let mut v = vec![];
        for stream in streams {
            let stream_str = serde_json::to_string(&stream)?;
            v.push(serde_json::from_str(&stream_str)?);
        }
        Ok(v)
    }
    fn audio_only_streams(&self) -> Result<Vec<StreamItem>, FieldError> {
        let streams = self.extractor.get_audio_streams()?;
        let mut v = vec![];
        for stream in streams {
            let stream_str = serde_json::to_string(&stream)?;
            v.push(serde_json::from_str(&stream_str)?);
        }
        Ok(v)
    }

    fn title(&self) -> Result<String, FieldError> {
        Ok(self.extractor.get_name()?)
    }

    fn description(&self) -> Result<String, FieldError> {
        Ok(self.extractor.get_description(false)?.0)
    }

    fn uploader_name(&self) -> Result<String, FieldError> {
        Ok(self.extractor.get_uploader_name()?)
    }

    fn uploader_url(&self) -> Result<String, FieldError> {
        Ok(self.extractor.get_uploader_url()?)
    }

    fn video_thumbnails(&self) -> Result<Vec<Thumbnail>, FieldError> {
        let thumbs = self.extractor.get_video_thumbnails()?;
        let mut thumbf = vec![];
        for thumb in thumbs {
            thumbf.push(Thumbnail {
                url: fix_thumbnail_url(&thumb.url),
                height: thumb.height as i32,
                width: thumb.width as i32,
            })
        }
        Ok(thumbf)
    }

    fn uploader_thumbnails(&self) -> Result<Vec<Thumbnail>, FieldError> {
        let thumbs = self.extractor.get_uploader_avatar_url()?;
        let mut thumbf = vec![];
        for thumb in thumbs {
            thumbf.push(Thumbnail {
                url: fix_thumbnail_url(&thumb.url),
                height: thumb.height as i32,
                width: thumb.width as i32,
            })
        }
        Ok(thumbf)
    }

    fn likes(&self) -> Result<i32, FieldError> {
        Ok(self.extractor.get_like_count()? as i32)
    }

    fn dislikes(&self) -> Result<i32, FieldError> {
        Ok(self.extractor.get_dislike_count()? as i32)
    }

    fn views(&self) -> Result<i32, FieldError> {
        Ok(self.extractor.get_view_count()? as i32)
    }

    fn length(&self) -> Result<i32, FieldError> {
        Ok(self.extractor.get_length()? as i32)
    }

    fn related(&self) -> Result<Vec<SearchResult>, FieldError> {
        let mut result = vec![];
        for item in self.extractor.get_related()? {
            result.push(match item {
                YTSearchItem::StreamInfoItem(vid) => SearchResult::VideoInfo(VideoResult {
                    name: vid.get_name()?,
                    video_id: vid.video_id()?,
                    is_ad: vid.is_ad().unwrap_or(false),
                    is_premium_video: vid.is_premium_video().unwrap_or(false),
                    url: vid.get_url()?,
                    is_live: vid.is_live().unwrap_or(false),
                    duration: vid.get_duration().ok(),
                    uploader_name: vid.get_uploader_name().ok(),
                    uploader_url: vid.get_uploader_url().ok(),
                    upload_date: vid.get_textual_upload_date().ok(),
                    view_count: vid.get_view_count().ok(),
                    thumbnail: vid
                        .get_thumbnails()?
                        .iter()
                        .map(|f| Thumbnail {
                            url: fix_thumbnail_url(&f.url),
                            width: f.width as i32,
                            height: f.height as i32,
                        })
                        .collect(),
                }),
                YTSearchItem::ChannelInfoItem(channel) => {
                    SearchResult::ChannelInfo(ChannelResult {
                        name: channel.get_name()?,
                        channel_id: channel.channel_id()?,
                        thumbnail: channel
                            .get_thumbnails()?
                            .iter()
                            .map(|f| Thumbnail {
                                url: fix_thumbnail_url(&f.url),
                                width: f.width as i32,
                                height: f.height as i32,
                            })
                            .collect(),
                        url: channel.get_url()?,
                        subscribers: channel.get_subscriber_count().ok(),
                        videos: channel.get_stream_count().ok(),
                        description: channel.get_description()?,
                    })
                }
                YTSearchItem::PlaylistInfoItem(playlist) => {
                    SearchResult::PlaylistInfo(PlaylistResult {
                        name: playlist.get_name()?,
                        playlist_id: playlist.playlist_id()?,
                        thumbnail: playlist
                            .get_thumbnails()?
                            .iter()
                            .map(|f| Thumbnail {
                                url: fix_thumbnail_url(&f.url),
                                width: f.width as i32,
                                height: f.height as i32,
                            })
                            .collect(),
                        url: playlist.get_url()?,
                        uploader_name: playlist.get_uploader_name().ok(),
                        videos: playlist.get_stream_count().ok(),
                    })
                }
            })
        }
        Ok(result)
    }


}

#[derive(juniper::GraphQLObject, Serialize, Deserialize)]
pub struct StreamItem {
    pub url: String,
    pub itag: i32,
    pub approxDurationMs: Option<String>,
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

#[derive(juniper::GraphQLObject, Serialize, Deserialize)]
pub struct Thumbnail {
    url: String,
    width: i32,
    height: i32,
}

struct Query;

#[juniper::graphql_object(Context = Context)]
impl Query {
    async fn video(video_id: String) -> Result<Video, FieldError> {
        let ytextractor = YTStreamExtractor::new(&video_id, DownloaderObj).await?;
        Ok(Video {
            extractor: ytextractor,
        })
    }

    async fn search(query: String, page_url: Option<String>) -> Result<Search, FieldError> {
        let extractor = YTSearchExtractor::new::<DownloaderObj>( &query, page_url).await?;
        Ok(Search { extractor })
    }

    async fn channel(channel_id: String, page_url: Option<String>) -> Result<Channel, FieldError> {
        let extractor = YTChannelExtractor::new::<DownloaderObj>(&channel_id,  page_url).await?;
        Ok(Channel { extractor })
    }

    async fn playlist(
        playlist_id: String,
        page_url: Option<String>,
    ) -> Result<Playlist, FieldError> {
        let extractor = YTPlaylistExtractor::new(&playlist_id, DownloaderObj, page_url).await?;
        Ok(Playlist { extractor })
    }

    async fn trending() -> Result<Trending, FieldError> {
        let extractor = YTTrendingExtractor::new(DownloaderObj).await?;
        Ok(Trending { extractor })
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

    let cors = warp::cors().allow_any_origin()
        .allow_methods(vec!["POST","GET"])
        .allow_headers(vec!["User-Agent", "Sec-Fetch-Mode","x-apollo-tracing","content-type", "Referer", "Origin", "Access-Control-Request-Method", "Access-Control-Request-Headers"])
    .build(); 

    warp::serve(
        warp::get()
            .and(warp::path("graphiql"))
            .and(juniper_warp::graphiql_filter("/graphql", None))
            .or(homepage)
            .or(warp::path("graphql").and(graphql_filter))
            .with(cors).with(log),
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
