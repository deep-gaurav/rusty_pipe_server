use juniper::FieldError;

use super::Context;
use rusty_pipe::youtube_extractor::search_extractor::{YTSearchItem, YTSearchExtractor};
use crate::Thumbnail;

pub struct Search {
    pub extractor: YTSearchExtractor
}
#[juniper::graphql_object(Context = Context)]
impl Search{

    fn suggestion(&self)->Result<String,FieldError>{
        Ok(self.extractor.get_search_suggestion()?)
    }

    fn result(&self)->Result<Vec<SearchResult>,FieldError>{
        let mut result = vec![];
        for item in self.extractor.collect_items()?{
            result.push(
                match item{
                    YTSearchItem::StreamInfoItem(vid) => {
                        SearchResult::VideoInfo(
                            VideoResult{
                                name: vid.get_name()?,
                                is_ad: vid.is_ad().unwrap_or(false),
                                is_premium_video: vid.is_premium_video().unwrap_or(false),
                                url: vid.get_url()?,
                                is_live: vid.is_live().unwrap_or(false),
                                duration: vid.get_duration().ok(),
                                uploader_name: vid.get_uploader_name().ok(),
                                uploader_url: vid.get_uploader_url().ok(),
                                upload_date: vid.get_textual_upload_date().ok(),
                                view_count: vid.get_view_count().ok(),
                                thumbnail:vid.get_thumbnails()?.iter().map(|f|Thumbnail{
                                    url:f.url.clone(),
                                    width: f.width as i32,
                                    height:f.height as i32
                                }).collect()
                            }
                        )
                    },
                    YTSearchItem::ChannelInfoItem(channel) => {
                        SearchResult::ChannelInfo(
                            ChannelResult{
                                name: channel.get_name()?,
                                thumbnail: channel.get_thumbnails()?.iter().map(|f|Thumbnail{
                                    url:f.url.clone(),
                                    width: f.width as i32,
                                    height:f.height as i32
                                }).collect(),
                                url: channel.get_url()?,
                                subscribers: channel.get_subscriber_count().ok(),
                                videos: channel.get_stream_count().ok(),
                                description: channel.get_description()?
                            }
                        )
                    },
                    YTSearchItem::PlaylistInfoItem(playlist) => {
                        SearchResult::PlaylistInfo(
                            PlaylistResult{
                                name: playlist.get_name()?,
                                thumbnail: playlist.get_thumbnails()?.iter().map(|f|Thumbnail{
                                    url:f.url.clone(),
                                    width: f.width as i32,
                                    height:f.height as i32
                                }).collect(),
                                url: playlist.get_url()?,
                                uploader_name: playlist.get_uploader_name().ok(),
                                videos: playlist.get_stream_count().ok()
                            }
                        )
                    },
                }
            )
        }
        Ok(result)
    }

}

#[derive(juniper::GraphQLObject)]
pub struct VideoResult{
    name:String,
    is_ad:bool,
    is_premium_video:bool,
    url:String,
    is_live:bool,
    duration:Option<i32>,
    uploader_name:Option<String>,
    uploader_url:Option<String>,
    upload_date:Option<String>,
    view_count:Option<i32>,
    thumbnail:Vec<Thumbnail>
}

#[derive(juniper::GraphQLObject)]
pub struct PlaylistResult{
    name:String,
    thumbnail:Vec<Thumbnail>,
    url:String,
    uploader_name:Option<String>,
    videos:Option<i32>
}

#[derive(juniper::GraphQLObject)]
pub struct ChannelResult{
    name:String,
    thumbnail:Vec<Thumbnail>,
    url:String,
    subscribers:Option<i32>,
    videos:Option<i32>,
    description:Option<String>
}

#[derive(juniper::GraphQLUnion)]
pub enum SearchResult{
    VideoInfo(VideoResult),
    PlaylistInfo(PlaylistResult),
    ChannelInfo(ChannelResult)
}