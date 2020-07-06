use crate::hn_client::{Comment, Post};
use std::collections::HashMap;

pub struct ViewState {
  pub page: Page,
  pub scroll_offset: u16,
}

pub enum Page {
  PostList { offset: u32, cursor_index: u32 },
  PostDetails { post: u32, cursor_index: u32 },
}

pub fn init_store() -> DataStore {
  DataStore {
    top_post_ids: Vec::new(),
    best_post_ids: Vec::new(),
    new_post_ids: Vec::new(),
    posts: HashMap::new(),
    comments: HashMap::new(),
  }
}

pub struct DataStore {
  pub top_post_ids: Vec<u32>,
  pub best_post_ids: Vec<u32>,
  pub new_post_ids: Vec<u32>,
  pub posts: HashMap<u32, Post>,
  pub comments: HashMap<u32, Comment>,
}

impl DataStore {
  
}
