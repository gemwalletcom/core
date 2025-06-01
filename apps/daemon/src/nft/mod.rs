// mod collections_updater;
// mod image_uploader;
// use collections_updater::OpenSeaUpdater;
// use image_uploader::ImageUploaderClient;
// use nft::OpenSeaClient;
// mod collections_image_uploader;
// use collections_image_uploader::CollectionsImageUploader;

// use job_runner::run_job;
use settings::Settings;
use std::future::Future;
use std::pin::Pin;
// use std::sync::Arc;
// use std::time::Duration;

pub async fn jobs(_settings: Settings) -> Vec<Pin<Box<dyn Future<Output = ()> + Send>>> {
    // let open_sea_collections_updater = run_job("Update collections", Duration::from_secs(3600), {
    //     let settings = Arc::new(settings.clone());

    //     move || {
    //         let opensea_client = OpenSeaClient::new(&settings.nft.opensea.key.secret);
    //         let mut updater = OpenSeaUpdater::new(&settings.postgres.url, opensea_client);
    //         async move { updater.update_collections().await }
    //     }
    // });

    // let collections_image_uploader = run_job("Upload collection images to R2 bucket", Duration::from_secs(3600), {
    //     let settings = Arc::new(settings.clone());
    //     move || {
    //         let bucket = settings.nft.bucket.clone();
    //         let image_uploader = ImageUploaderClient::new(bucket.clone());
    //         let mut updater = CollectionsImageUploader::new(settings.postgres.url.as_str(), image_uploader);
    //         async move { updater.update_collections().await }
    //     }
    // });

    // let collection_assets_image_uploader = run_job("Upload collection assets images to R2 bucket", Duration::from_secs(3600), {
    //     let settings = Arc::new(settings.clone());
    //     move || {
    //         let bucket = settings.nft.bucket.clone();
    //         let image_uploader = ImageUploaderClient::new(bucket.clone());
    //         let mut updater = CollectionsImageUploader::new(settings.postgres.url.as_str(), image_uploader);
    //         async move { updater.update_collection_assets().await }
    //     }
    // });

    vec![
        //Box::pin(open_sea_collections_updater),
        // Box::pin(collections_image_uploader),
        // Box::pin(collection_assets_image_uploader),
    ]
}
