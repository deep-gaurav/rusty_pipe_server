use crate::search::VideoResult;
use crate::Context;
use crate::Thumbnail;
use juniper::FieldError;
use rusty_pipe::utils::utils::fix_thumbnail_url;
use rusty_pipe::youtube_extractor::channel_extractor::YTChannelExtractor;

pub struct Channel {
    pub extractor: YTChannelExtractor,
}

#[juniper::graphql_object(Context = Context)]
impl Channel {
    fn name(&self) -> Result<String, FieldError> {
        Ok(self.extractor.get_name()?)
    }

    fn avatars(&self) -> Result<Vec<Thumbnail>, FieldError> {
        let mut avatars = vec![];
        for avatar in self.extractor.get_avatars()? {
            avatars.push(Thumbnail {
                url: fix_thumbnail_url(&avatar.url),
                width: avatar.width as i32,
                height: avatar.height as i32,
            })
        }
        Ok(avatars)
    }

    fn banners(&self) -> Result<Vec<Thumbnail>, FieldError> {
        let mut banners = vec![];
        for banner in self.extractor.get_banners()? {
            banners.push(Thumbnail {
                url: fix_thumbnail_url(&banner.url),
                width: banner.width as i32,
                height: banner.height as i32,
            })
        }
        Ok(banners)
    }

    fn next_page_url(&self) -> Result<Option<String>, FieldError> {
        Ok(self.extractor.get_next_page_url()?)
    }

    fn videos(&self) -> Result<Vec<VideoResult>, FieldError> {
        let mut videos = vec![];
        for vid in self.extractor.get_videos()? {
            videos.push(VideoResult {
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
