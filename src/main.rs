// mod channel;
// mod lib;
// mod playlist;
// mod search;
// mod trending;
// mod vidproxy;

use serde::{Deserialize, Serialize};

use async_trait::async_trait;
use rusty_pipe::downloader_trait::Downloader;
use rusty_pipe::youtube_extractor::search_extractor::{YTSearchExtractor, YTSearchItem};
use rusty_pipe::youtube_extractor::stream_extractor::YTStreamExtractor;

use juniper::{EmptyMutation, EmptySubscription, FieldError, RootNode};
use rusty_pipe::youtube_extractor::error::ParsingError;
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::str::FromStr;
use warp::{http::Response, Filter};

use lazy_static::lazy_static;
use rusty_pipe::utils::utils::fix_thumbnail_url;
use rusty_pipe::youtube_extractor::channel_extractor::YTChannelExtractor;
use rusty_pipe::youtube_extractor::playlist_extractor::YTPlaylistExtractor;
use rusty_pipe::youtube_extractor::trending_extractor::YTTrendingExtractor;
use rusty_pipe_server::channel::Channel;
use rusty_pipe_server::playlist::Playlist;
use rusty_pipe_server::trending::Trending;
use rusty_pipe_server::vidproxy::vidproxyhandle;

use rusty_pipe_server::search::*;

use rusty_pipe_server::*;

#[tokio::main]
async fn main() {
    ::std::env::set_var("RUST_LOG", "warp_async");
    env_logger::init();
    run_server_async(
        [0, 0, 0, 0],
        std::env::var("PORT")
            .unwrap_or("8080".to_owned())
            .parse()
            .unwrap(),
    )
    .await;
}
