mod hn_client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Fetch top 20 + get data
    let mut results = hn_client::get_stories(hn_client::StoryListType::Top, 0, 20).await?;
    let a = results[0].children;
    a.load_children().await;
    match a {
        hn_client::Children::Loaded(b) => {b},
        _ => {panic!("bah!")}
    };
    Ok(())
}
