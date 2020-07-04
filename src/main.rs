mod hn_client;
use hn_client::{Post, Comment};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut top_posts: Vec<u32> = Vec::new();
    let mut post_hash: HashMap<u32, Post> = HashMap::new();
    let mut comment_hash: HashMap<u32, Comment> = HashMap::new();

    let posts = hn_client::get_stories(hn_client::StoryListType::Top, 0, 20).await?;
    for post in posts {
        top_posts.push(post.id);
        post_hash.insert(post.id, post);
    }

    if top_posts.len() > 0 {
        let id = top_posts[1];
        let post = post_hash.get(&id).unwrap();
        let comments = hn_client::get_comments(&post.children).await?;

        for comment in comments {
            comment_hash.insert(comment.id, comment);
        }
    }

    // Render list of posts here
    // get the top 10

    println!("{:?}", post_hash);
    println!("{:?}", comment_hash);
    Ok(())
}
