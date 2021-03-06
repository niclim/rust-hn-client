use crate::stores::view::StoryListType;
use std::collections::HashMap;

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

pub struct Comment {
    pub id: u32,
    pub by: String,
    pub children: Vec<u32>,
    pub parent: u32,
    pub text: String,
    pub time: u32,
}

pub struct DataStore {
    top_post_ids: Vec<u32>,
    best_post_ids: Vec<u32>,
    new_post_ids: Vec<u32>,
    posts: HashMap<u32, Post>,
    pub comments: HashMap<u32, Comment>,
}

impl DataStore {
    pub fn init() -> DataStore {
        DataStore {
            top_post_ids: Vec::new(),
            best_post_ids: Vec::new(),
            new_post_ids: Vec::new(),
            posts: HashMap::new(),
            comments: HashMap::new(),
        }
    }

    pub fn has_post_ids(&self, story_type: &StoryListType) -> bool {
        match story_type {
            StoryListType::Top => self.top_post_ids.len() > 0,
            StoryListType::Best => self.best_post_ids.len() > 0,
            StoryListType::New => self.new_post_ids.len() > 0,
        }
    }

    pub fn get_post_ids(&self, story_type: &StoryListType) -> &[u32] {
        match story_type {
            StoryListType::Top => &self.top_post_ids,
            StoryListType::Best => &self.best_post_ids,
            StoryListType::New => &self.new_post_ids,
        }
    }

    pub fn hydrate_post_ids(&mut self, story_type: &StoryListType, post_ids: Vec<u32>) {
        match story_type {
            StoryListType::Top => self.top_post_ids = post_ids,
            StoryListType::Best => self.best_post_ids = post_ids,
            StoryListType::New => self.new_post_ids = post_ids,
        };
    }

    pub fn get_post(&self, post_id: &u32) -> Option<&Post> {
        self.posts.get(post_id)
    }

    pub fn hydrate_posts(&mut self, posts: Vec<Post>) {
        for post in posts {
            self.posts.insert(post.id, post);
        }
    }

    pub fn get_comment(&self, comment_id: &u32) -> Option<&Comment> {
        self.comments.get(comment_id)
    }

    pub fn hydrate_comments(&mut self, comments: Vec<Comment>) {
        for comment in comments {
            self.comments.insert(comment.id, comment);
        }
    }

    pub fn get_missing_post_ids(&self, post_ids: &[u32]) -> Vec<u32> {
        post_ids
            .iter()
            .cloned()
            // TODO handle error checking when add errors into hashmap
            .filter(|post_id| !self.posts.contains_key(post_id))
            .collect()
    }

    pub fn get_missing_comment_ids(&self, comment_ids: &[u32]) -> Vec<u32> {
        comment_ids
            .iter()
            .cloned()
            // TODO handle error checking when add errors into hashmap
            .filter(|comment_id| !self.comments.contains_key(comment_id))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::distributions::Alphanumeric;
    use rand::Rng;

    fn make_post(id: u32) -> Post {
        let mut rng = rand::thread_rng();
        Post {
            id: id,
            by: rng.sample_iter(&Alphanumeric).take(30).collect(),
            children: vec![0; 5].iter().map(|_| rng.gen()).collect(),
            title: rng.sample_iter(&Alphanumeric).take(30).collect(),
            time: rng.gen(),
            url: Some(rng.sample_iter(&Alphanumeric).take(30).collect()),
            text: Some(rng.sample_iter(&Alphanumeric).take(30).collect()),
            descendants: rng.gen(),
        }
    }

    fn make_comment(id: u32) -> Comment {
        let mut rng = rand::thread_rng();
        Comment {
            id: id,
            by: rng.sample_iter(&Alphanumeric).take(30).collect(),
            children: vec![0; 5].iter().map(|_| rng.gen()).collect(),
            parent: rng.gen(),
            text: rng.sample_iter(&Alphanumeric).take(30).collect(),
            time: rng.gen(),
        }
    }

    #[test]
    fn data_store_posts() {
        let mut data_store = DataStore::init();
        let n_posts = 5;
        let mock_posts: Vec<Post> = (0..n_posts)
            .collect::<Vec<u32>>()
            .into_iter()
            .map(|id| make_post(id))
            .collect();

        for enum_variant in vec![StoryListType::Best, StoryListType::Top, StoryListType::New].iter()
        {
            assert_eq!(data_store.has_post_ids(&enum_variant), false);
            assert_eq!(data_store.get_post_ids(&enum_variant).len(), 0);
        }

        // hydrate store
        let post_ids_to_hydrate: Vec<u32> = mock_posts.iter().map(|post| post.id).collect();
        data_store.hydrate_post_ids(&StoryListType::Best, post_ids_to_hydrate);
        data_store.hydrate_posts(mock_posts);

        assert_eq!(data_store.has_post_ids(&StoryListType::Best), true);
        assert_eq!(
            data_store.get_post_ids(&StoryListType::Best).len() as u32,
            n_posts
        );

        let post_ids = data_store.get_post_ids(&StoryListType::Best);
        for post_id in post_ids {
            // Should have a valid post stored
            data_store.get_post(&post_id).unwrap();
        }
    }

    #[test]
    fn data_store_comments() {
        let mut data_store = DataStore::init();
        let comment_ids: Vec<u32> = (0..5).collect();
        let mock_comments: Vec<Comment> = comment_ids
            .iter()
            .cloned()
            .map(|id| make_comment(id))
            .collect();
        data_store.hydrate_comments(mock_comments);

        for comment_id in comment_ids {
            // Should have a valid post stored
            data_store.get_comment(&comment_id).unwrap();
        }
    }

    #[test]
    fn missing_post_and_comments() {
        let mut data_store = DataStore::init();
        let post_ids: Vec<u32> = (0..5).collect();
        let comment_ids: Vec<u32> = (6..10).collect();
        let mock_posts: Vec<Post> = post_ids.iter().cloned().map(|id| make_post(id)).collect();
        let mock_comments: Vec<Comment> = comment_ids
            .iter()
            .cloned()
            .map(|id| make_comment(id))
            .collect();
        data_store.hydrate_posts(mock_posts);
        data_store.hydrate_comments(mock_comments);

        let missing_post_ids = data_store.get_missing_post_ids(&(0..6).collect::<Vec<u32>>());
        let missing_comment_ids =
            data_store.get_missing_comment_ids(&(6..11).collect::<Vec<u32>>());
        println!("{:?}", missing_post_ids);
        assert_eq!(missing_post_ids.len(), 1);
        assert_eq!(missing_comment_ids.len(), 1);
        assert_eq!(missing_post_ids[0], 5);
        assert_eq!(missing_comment_ids[0], 10);
    }
}
