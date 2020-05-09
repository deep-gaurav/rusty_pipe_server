use juniper::FieldError;

use super::Context;
use rusty_pipe::youtube_extractor::search_extractor::{YTSearchItem, YTSearchExtractor};

pub struct Search {
    pub extractor: YTSearchExtractor
}
#[juniper::graphql_object(Context = Context)]
impl Search{

    fn suggestion(&self)->Option<String>{
        self.extractor.get_search_suggestion()
    }

    fn result(&self)->Result<Vec<SearchResult>,FieldError>{
        let mut result = vec![];
        for item in self.extractor.collect_items(){
            result.push(
                match item{
                    YTSearchItem::StreamInfoItem(vid) => {
                        SearchResult::VideoInfo(
                            VideoResult{
                                name: vid.get_name().ok_or("Cant get name")?.to_string(),
                                is_ad: vid.is_ad(),
                                is_premium_video: vid.is_premium_video(),
                                url: vid.get_url().ok_or("Cant get url")?.to_string(),
                                is_live: vid.is_live(),
                                duration: vid.get_duration().map(|f|f as i32),
                                uploader_name: vid.get_uploader_name().map(|f|f.to_string()),
                                uploader_url: vid.get_uploader_url(),
                                upload_date: vid.get_textual_upload_date().map(|f|f.to_string()),
                                view_count: vid.get_view_count().map(|f|f as i32),
                                thumbnail_url: vid.get_thumbnail_url()
                            }
                        )
                    },
                    YTSearchItem::ChannelInfoItem(channel) => {
                        SearchResult::ChannelInfo(
                            ChannelResult{
                                name: channel.get_name().ok_or("Cant get name")?.to_string(),
                                thumbnail_url: channel.get_thumbnail_url().map(|f|f.to_string()),
                                url: channel.get_url().ok_or("Cant get url")?.to_string(),
                                subscribers: channel.get_subscriber_count().map(|f|f as i32),
                                videos: channel.get_stream_count().map(|f|f as i32),
                                description: channel.get_description().map(|f|f.to_string())
                            }
                        )
                    },
                    YTSearchItem::PlaylistInfoItem(playlist) => {
                        SearchResult::PlaylistInfo(
                            PlaylistResult{
                                name: playlist.get_name().ok_or("Cant get name")?.to_string(),
                                thumbnail_url: playlist.get_thumbnail_url().map(|f|f.to_string()),
                                url: playlist.get_url().ok_or("Cant get url")?.to_string(),
                                uploader_name: playlist.get_uploader_name().map(|f|f.to_string()),
                                videos: playlist.get_stream_count().map(|f|f as i32)
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
    thumbnail_url:Option<String>
}

#[derive(juniper::GraphQLObject)]
pub struct PlaylistResult{
    name:String,
    thumbnail_url:Option<String>,
    url:String,
    uploader_name:Option<String>,
    videos:Option<i32>
}

#[derive(juniper::GraphQLObject)]
pub struct ChannelResult{
    name:String,
    thumbnail_url:Option<String>,
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