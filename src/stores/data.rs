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
    pub posts: HashMap<u32, Post>,
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

    pub fn hydrate_posts(&mut self, posts: Vec<Post>) {
        for post in posts {
            self.posts.insert(post.id, post);
        }
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
            .filter(|post_id| self.posts.contains_key(post_id))
            .collect()
    }

    pub fn get_missing_comment_ids(&self, comment_ids: &[u32]) -> Vec<u32> {
        comment_ids
            .iter()
            .cloned()
            // TODO handle error checking when add errors into hashmap
            .filter(|comment_id| self.comments.contains_key(comment_id))
            .collect()
    }
}
