use crate::search::VideoResult;
use crate::Thumbnail;
use rusty_pipe::utils::utils::fix_thumbnail_url;
use rusty_pipe::youtube_extractor::error::ParsingError;
use rusty_pipe::youtube_extractor::trending_extractor::YTTrendingExtractor;
use crate::Context;
use juniper::FieldError;

pub struct Trending {
    pub extractor: YTTrendingExtractor,
}

#[juniper::graphql_object(Context = Context)]
impl Trending {
    fn videos(&self) -> Result<Vec<VideoResult>, FieldError> {
        let mut videos = vec![];
        for vid in self.extractor.get_videos()? {
            videos.push(VideoResult {
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
            });
        }
        Ok(videos)
    }
}
