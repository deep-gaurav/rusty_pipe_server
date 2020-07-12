use crate::DownloaderObj;
use rusty_pipe::youtube_extractor::error::ParsingError;
use rusty_pipe::youtube_extractor::stream_extractor::YTStreamExtractor;
use warp::Filter;


pub async fn vidproxyhandle(video_id: String, itag: u32) -> Result<impl warp::Reply, warp::Rejection> {
    let videx = YTStreamExtractor::new(&video_id, DownloaderObj)
        .await
        .map_err(|e| {eprintln!("stream not created");warp::reject::not_found()})?;
    let mut streams = videx.get_video_only_stream().unwrap_or_default();
    streams.append(&mut videx.get_audio_streams().unwrap_or_default());
    let reqstream = streams.iter().find(|st| st.itag == itag);
    match reqstream {
        Some(reqstream) => {
            use hyper_tls::HttpsConnector;

            // use warp::hyper::Client;
            let https = HttpsConnector::new();
            let http_client =  hyper::Client::builder().build::<_, warp::hyper::Body>(https);
            use std::str::FromStr;
            use warp::hyper::Uri;
            
            println!("Serving url {:#?}",reqstream.url);
            let resp = http_client
                .get(Uri::from_str(reqstream.url.as_ref().unwrap_or(&"".to_string())).unwrap())
                .await
                .map_err(|e| {eprintln!("http error {:#}",e);warp::reject::not_found()})?;
            println!("Success {:#?}",resp);
            
            let location = resp.headers().get(hyper::header::LOCATION);
            if let Some(location)= location{
                println!("following redirect {:#?}",location);
                let resp = http_client
                .get(Uri::from_str(location.to_str().unwrap()).unwrap())
                .await
                .map_err(|e| {eprintln!("http error {:#}",e);warp::reject::not_found()})?;
                println!("Success {:#?}",resp);
                return Ok(resp)
            }
            Ok(resp)
        }
        None =>{eprintln!("No stream of itag {}",itag); Err(warp::reject::not_found())},
    }
}
