use crate::error::AppResult;
use crate::news::{self, NewsItem};
use crate::state::AppState;
use tauri::State;

#[tauri::command]
pub async fn fetch_news(state: State<'_, AppState>) -> AppResult<Vec<NewsItem>> {
    news::fetch(&state.client).await
}
