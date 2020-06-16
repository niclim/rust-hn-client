mod hn_client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Fetch top 20 + get data
    let results = hn_client::get_stories(hn_client::StoryListType::Top, 0, 20).await?;
    println!("{:?}", results);
    Ok(())
}
