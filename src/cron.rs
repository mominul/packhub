use crate::{platform::AptPlatformDetection, state::AppState};

pub async fn run_cron_job(state: AppState) {
    let db = state.db().database("repology");
    let collection = db.collection::<AptPlatformDetection>("apt");

    AptPlatformDetection::update(&collection).await;
}
