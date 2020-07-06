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
  pub top_post_ids: Vec<u32>,
  pub best_post_ids: Vec<u32>,
  pub new_post_ids: Vec<u32>,
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
}
