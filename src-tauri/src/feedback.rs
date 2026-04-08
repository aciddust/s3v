use serde::Serialize;

const FEEDBACK_URL: &str = "https://web.d3fau1t.net/api/feedback";
const ORIGIN: &str = "https://aciddust.github.io";

#[derive(Serialize)]
struct FeedbackBody {
    name: String,
    email: String,
    content: String,
}

#[tauri::command]
pub async fn send_feedback(name: String, email: String, content: String) -> Result<(), String> {
    let client = reqwest::Client::new();
    let res = client
        .post(FEEDBACK_URL)
        .header("Content-Type", "application/json")
        .header("Origin", ORIGIN)
        .header("Referer", format!("{}/", ORIGIN))
        .json(&FeedbackBody { name, email, content })
        .send()
        .await
        .map_err(|e| e.to_string())?;

    if res.status().is_success() {
        Ok(())
    } else {
        let status = res.status().as_u16();
        let body = res.text().await.unwrap_or_default();
        Err(format!("{}: {}", status, body))
    }
}
