use futures::{future, stream, StreamExt};
use reqwest::get;
use serde::{Deserialize, Serialize};

#[cfg(test)]
use mockito;

#[cfg(not(test))]
fn get_hn_url() -> String {
    String::from("https://hacker-news.firebaseio.com/v0")
}

#[cfg(test)]
fn get_hn_url() -> String {
    let url = mockito::server_url();
    url
}

const PARALLEL_REQUESTS: usize = 5;

enum Route {
    New,
    Top,
    Best,
    // Ask,
    // Job,
    // Show,
    Item,
}

#[derive(Debug)]
pub struct Post {
    pub id: u32,
    pub by: String,
    pub children: Vec<u32>,
    pub title: String,
    pub time: u32,
    pub url: Option<String>,
    pub text: Option<String>,
    pub descendants: u32,
}

#[derive(Debug)]
pub struct Comment {
    pub id: u32,
    pub by: String,
    pub children: Vec<u32>,
    pub parent: u32,
    pub text: String,
    pub time: u32,
}

// Defined by https://github.com/HackerNews/API#items
#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum Item {
    Job(HnJob),
    Story(HnStory),
    Comment(HnComment),
    Poll,
    Pollopt,
}

// A story can be an Ask or Story
// Ask -> Some(String)
// Story -> None
#[derive(Debug, Deserialize, Serialize, Clone)]
struct HnStory {
    by: String,
    descendants: u32,
    id: u32,
    kids: Vec<u32>,
    text: Option<String>,
    score: u16,
    time: u32,
    title: String,
    url: Option<String>,
}

#[derive(Debug, Deserialize, Serialize)]
struct HnJob {
    by: String,
    id: u32,
    score: u16,
    text: String,
    time: u32,
    title: String,
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct HnComment {
    by: String,
    id: u32,
    kids: Vec<u32>,
    parent: u32,
    text: String,
    time: u32,
}

fn get_route(route: Route) -> String {
    let path = match route {
        Route::New => "/newstories.json",
        Route::Top => "/topstories.json",
        Route::Best => "/beststories.json",
        // Route::Ask => "/askstories.json",
        // Route::Job => "/jobstories.json",
        // Route::Show => "/showstories.json",
        Route::Item => "/item",
    };
    let base_hn_url = get_hn_url();
    return format!("{base_url}{path}", base_url = base_hn_url, path = path);
}

fn get_item_route(id: &u32) -> String {
    let base_url = get_route(Route::Item);
    return format!("{base_url}/{id}.json", base_url = base_url, id = id);
}

async fn get_item(id: &u32) -> Result<Item, Box<dyn std::error::Error>> {
    let route = get_item_route(id);
    let body = get(&route).await?.json().await?;
    Ok(body)
}

async fn get_top_post_ids() -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    let route = get_route(Route::Top);
    // This request returns the top 500 stories
    // This _could_ be cached
    let body: Vec<u32> = get(&route).await?.json().await?;
    Ok(body)
}

async fn get_best_post_ids() -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    let route = get_route(Route::Best);
    // This request returns the top 500 stories
    // This _could_ be cached
    let body: Vec<u32> = get(&route).await?.json().await?;
    Ok(body)
}

async fn get_new_post_ids() -> Result<Vec<u32>, Box<dyn std::error::Error>> {
    let route = get_route(Route::New);
    // This request returns the top 500 stories
    // This _could_ be cached
    let body: Vec<u32> = get(&route).await?.json().await?;
    Ok(body)
}

async fn get_items(ids: &[u32]) -> Vec<Item> {
    stream::iter(ids)
        .map(|item_id| async move { get_item(item_id).await })
        .buffer_unordered(PARALLEL_REQUESTS)
        // TODO - handle error messaging / logging
        // Right now errors are silently swallowed
        .filter(|item_response| future::ready(item_response.is_ok()))
        .map(|item_response| item_response.unwrap())
        .collect::<Vec<Item>>()
        .await
}

pub enum StoryListType {
    New,
    Best,
    Top,
}

// Returns top 500 stories - also contains jobs
pub async fn get_stories(
    story_type: StoryListType,
    skip: usize,
    limit: usize,
) -> Result<Vec<Post>, Box<dyn std::error::Error>> {
    let post_ids: Vec<u32> = match story_type {
        StoryListType::Top => get_top_post_ids().await?,
        StoryListType::Best => get_best_post_ids().await?,
        StoryListType::New => get_new_post_ids().await?,
    };

    let paginated_post_ids = &post_ids[skip..skip+limit];

    let posts_bodies = get_items(paginated_post_ids)
        .await
        .into_iter()
        .filter(|item| match item {
            Item::Story(_) => true,
            _ => false,
        })
        // Coerse item -> public facing Post struct
        .map(|item| match item {
            Item::Story(story) => Post {
                id: story.id,
                by: story.by,
                children: story.kids,
                title: story.title,
                time: story.time,
                url: story.url,
                text: story.text,
                descendants: story.descendants,
            },
            _ => panic!("Unexpected Item variant"),
        })
        .collect::<Vec<Post>>();

    Ok(posts_bodies)
}

pub async fn get_comments(children: &[u32]) -> Result<Vec<Comment>, Box<dyn std::error::Error>> {
    // TODO add some sort of limit here with children
    // (i.e. don't load all children if great than x)
    // will probably need to change the childrens enum
    let comment_bodies = get_items(children)
        .await
        .into_iter()
        .filter(|item| match item {
            Item::Comment(_) => true,
            _ => false,
        })
        // // Coerse item -> public facing Post struct
        .map(|item| match item {
            Item::Comment(comment) => Comment {
                id: comment.id,
                by: comment.by,
                children: comment.kids,
                parent: comment.parent,
                text: comment.text,
                time: comment.time,
            },
            _ => panic!("Unexpected Item variant"),
        })
        .collect::<Vec<Comment>>();

    Ok(comment_bodies)
}

#[cfg(test)]
mod tests {
    use super::*;
    use mockito::mock;
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    use serde_json;
    use std::collections::HashMap;

    fn make_mock_story(id: u32) -> Item {
        let mut rng = rand::thread_rng();
        Item::Story(HnStory {
            id: id,
            descendants: 5,
            by: rng.sample_iter(&Alphanumeric).take(10).collect(),
            kids: vec![0; 5].iter().map(|_| rng.gen()).collect(),
            text: Some(rng.sample_iter(&Alphanumeric).take(30).collect()),
            score: rng.gen(),
            time: rng.gen(),
            title: rng.sample_iter(&Alphanumeric).take(30).collect(),
            url: Some(rng.sample_iter(&Alphanumeric).take(10).collect()),
        })
    }

    fn make_mock_comment(id: u32, parent: u32) -> Item {
        let mut rng = rand::thread_rng();
        Item::Comment(HnComment {
            id: id,
            by: rng.sample_iter(&Alphanumeric).take(10).collect(),
            kids: vec![0; 5].iter().map(|_| rng.gen()).collect(),
            parent: parent,
            text: rng.sample_iter(&Alphanumeric).take(30).collect(),
            time: rng.gen(),
        })
    }

    #[tokio::test]
    async fn get_top_stories_returns_posts() {
        let story_ids: Vec<u32> = (0..30).collect();
        let raw_story_ids = serde_json::to_string(&story_ids).unwrap();
        let skip: usize = 5;
        let limit: usize = 20;

        let get_top_stories_mock = mock("GET", "/topstories.json")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(raw_story_ids)
            .expect(1)
            .create();

        let mock_stories: HashMap<u32, (Item, mockito::Mock)> = story_ids
            .iter()
            .cloned()
            .map(|id| {
                let route = format!("/item/{}.json", id).to_owned();
                let mock_story = make_mock_story(id);
                let raw_story = serde_json::to_string(&mock_story).unwrap();
                let expected_calls = if id >= skip as u32 && id < (skip + limit) as u32 {
                    1
                } else {
                    0
                };
                let mock_inst = mock("GET", &*route)
                    .with_status(200)
                    .with_header("content-type", "application/json")
                    .with_body(raw_story)
                    .expect(expected_calls)
                    .create();

                return (id, (mock_story, mock_inst));
            })
            .collect();

        let stories_result = get_stories(StoryListType::Top, skip, limit).await.unwrap();
        assert_eq!(stories_result.len(), limit);
        get_top_stories_mock.assert();

        for post in stories_result {
            let id = post.id;
            let (item_mock, mock_inst) = mock_stories.get(&id).unwrap();
            // Assert pagination params respected
            assert_eq!(true, id >= skip as u32 && id < (skip + limit) as u32);
            mock_inst.assert();

            match item_mock {
                Item::Story(story) => {
                    // Assert some key in object is parsed that we have a correct ref
                    assert_eq!(post.by, story.by);
                }
                _ => panic!("Unexpected Item variant"),
            }
        }
    }

    #[tokio::test]
    async fn get_comments_returns_comments() {
        let mut rng = rand::thread_rng();
        let parent_id: u32 = rng.gen();
        let comment_ids_to_get: Vec<u32> = vec![0; 5].iter().map(|_| rng.gen()).collect();
        let comment_ids_to_not_get: Vec<u32> = vec![0; 5].iter().map(|_| rng.gen()).collect();
        let all_comment_ids: Vec<u32> =
            [&comment_ids_to_get[..], &comment_ids_to_not_get[..]].concat();

        let mock_comments: HashMap<u32, (Item, mockito::Mock)> = all_comment_ids
            .iter()
            .cloned()
            .map(|id| {
                let route = format!("/item/{}.json", id).to_owned();
                let mock_comment = make_mock_comment(id, parent_id);
                let raw_comment = serde_json::to_string(&mock_comment).unwrap();
                // Yes this is O(n2) - however this is a small number
                let expected_calls = if comment_ids_to_get.contains(&id) {
                    1
                } else {
                    0
                };
                let mock_inst = mock("GET", &*route)
                    .with_status(200)
                    .with_header("content-type", "application/json")
                    .with_body(raw_comment)
                    .expect(expected_calls)
                    .create();

                return (id, (mock_comment, mock_inst));
            })
            .collect();

        let comments_result = get_comments(&comment_ids_to_get).await.unwrap();
        assert_eq!(comments_result.len(), comment_ids_to_get.len());
        for comment in comments_result {
            let (item_mock, mock_inst) = mock_comments.get(&comment.id).unwrap();
            mock_inst.assert();
            match item_mock {
                Item::Comment(comment_mock) => {
                    // Assert some key in object is parsed that we have a correct ref
                    assert_eq!(comment.by, comment_mock.by);
                }
                _ => panic!("Unexpected Item variant"),
            }
        }
    }
}
