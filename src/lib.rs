pub mod channel;
pub mod playlist;
pub mod search;
pub mod serverrunner;
pub mod trending;
pub mod vidproxy;

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

use crate::channel::Channel;
use crate::playlist::Playlist;
use crate::trending::Trending;
use crate::vidproxy::vidproxyhandle;
use lazy_static::lazy_static;
use rusty_pipe::utils::utils::fix_thumbnail_url;
use rusty_pipe::youtube_extractor::channel_extractor::YTChannelExtractor;
use rusty_pipe::youtube_extractor::playlist_extractor::YTPlaylistExtractor;
use rusty_pipe::youtube_extractor::trending_extractor::YTTrendingExtractor;
use serverrunner::*;

pub async fn run_server_async(port_ip: [u8; 4], port: u16) {
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

    let cors = warp::cors()
        .allow_any_origin()
        .allow_methods(vec!["POST", "GET"])
        .allow_headers(vec![
            "User-Agent",
            "Sec-Fetch-Mode",
            "x-apollo-tracing",
            "content-type",
            "Referer",
            "Origin",
            "Access-Control-Request-Method",
            "Access-Control-Request-Headers",
        ])
        .build();
    let vidproxy = warp::path!("vid" / String / u32).and_then(vidproxyhandle);

    warp::serve(
        warp::get()
            .and(warp::path("graphiql"))
            .and(juniper_warp::graphiql_filter("/graphql", None))
            .or(homepage)
            .or(warp::path("graphql").and(graphql_filter))
            .or(vidproxy)
            .with(cors)
            .with(log),
    )
    .run((port_ip, port))
    .await
}
