use juniper::FieldError;

use super::Context;
use crate::{DownloaderObj, Thumbnail};
use rusty_pipe::utils::utils::fix_thumbnail_url;
use rusty_pipe::youtube_extractor::search_extractor::{YTSearchExtractor, YTSearchItem};

pub struct Search {
    pub extractor: YTSearchExtractor,
}
#[juniper::graphql_object(Context = Context)]
impl Search {
    async fn suggestion(&self) -> Result<Vec<String>, FieldError> {
        Ok(YTSearchExtractor::get_search_suggestion::<DownloaderObj>("").await.map_err(|e|format!("{:#?}",e))?)
    }

    fn result(&self) -> Result<Vec<SearchResult>, FieldError> {
        let mut result = vec![];
        for item in self.extractor.search_results()? {
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

    fn next_page_url(&self) -> Result<Option<String>, FieldError> {
        Ok(self.extractor.get_next_page_url()?)
    }
}

#[derive(juniper::GraphQLObject)]
pub struct VideoResult {
    pub name: String,
    pub video_id: String,
    pub is_ad: bool,
    pub is_premium_video: bool,
    pub url: String,
    pub is_live: bool,
    pub duration: Option<i32>,
    pub uploader_name: Option<String>,
    pub uploader_url: Option<String>,
    pub upload_date: Option<String>,
    pub view_count: Option<i32>,
    pub thumbnail: Vec<Thumbnail>,
}

#[derive(juniper::GraphQLObject)]
pub struct PlaylistResult {
    name: String,
    playlist_id: String,
    thumbnail: Vec<Thumbnail>,
    url: String,
    uploader_name: Option<String>,
    videos: Option<i32>,
}

#[derive(juniper::GraphQLObject)]
pub struct ChannelResult {
    name: String,
    channel_id: String,
    thumbnail: Vec<Thumbnail>,
    url: String,
    subscribers: Option<i32>,
    videos: Option<i32>,
    description: Option<String>,
}

#[derive(juniper::GraphQLUnion)]
pub enum SearchResult {
    VideoInfo(VideoResult),
    PlaylistInfo(PlaylistResult),
    ChannelInfo(ChannelResult),
}
