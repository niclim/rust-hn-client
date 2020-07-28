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

pub struct PostCommentIds {
    i: usize,
    comment_ids: Vec<u32>,
}

impl Iterator for PostCommentIds {
    type Item = u32;

    fn next(&mut self) -> Option<u32> {
        match self.i >= self.comment_ids.len() {
            true => None,
            false => {
                let current_index = self.i;
                self.i = self.i + 1;
                Some(self.comment_ids[current_index])
            }
        }
    }
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

    fn _post_comment_helper(&self, comment_ids: Vec<u32>) -> Vec<u32> {
        comment_ids
            .iter()
            .flat_map(|comment_id| {
                let comment_children =
                    self._post_comment_helper(match self.get_comment(comment_id) {
                        Some(comment) => comment.children.iter().cloned().collect(),
                        None => Vec::new(),
                    });

                [&[*comment_id], &comment_children[..]].concat()
            })
            .collect::<Vec<u32>>()
    }

    // For now, we are going to ignore unloaded comments
    pub fn get_post_comments(&self, post_id: &u32) -> PostCommentIds {
        let comment_ids: Vec<u32> = self._post_comment_helper(
            self.get_post(post_id)
                .unwrap()
                .children
                .iter()
                .cloned()
                .collect(),
        );

        PostCommentIds {
            i: 0,
            comment_ids: comment_ids,
        }
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

    fn make_comment(id: u32, parent_id: u32) -> Comment {
        let mut rng = rand::thread_rng();
        Comment {
            id: id,
            by: rng.sample_iter(&Alphanumeric).take(30).collect(),
            children: vec![0; 5].iter().map(|_| rng.gen()).collect(),
            parent: parent_id,
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
            .map(|id| make_comment(id, 10))
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
            .map(|id| make_comment(id, 0))
            .collect();
        data_store.hydrate_posts(mock_posts);
        data_store.hydrate_comments(mock_comments);

        let missing_post_ids = data_store.get_missing_post_ids(&(0..6).collect::<Vec<u32>>());
        let missing_comment_ids =
            data_store.get_missing_comment_ids(&(6..11).collect::<Vec<u32>>());
        assert_eq!(missing_post_ids.len(), 1);
        assert_eq!(missing_comment_ids.len(), 1);
        assert_eq!(missing_post_ids[0], 5);
        assert_eq!(missing_comment_ids[0], 10);
    }

    #[test]
    fn post_comments_iterator() {
        fn make_comment_with_children(id: u32, parent_id: u32, children: Vec<u32>) -> Comment {
            let mut comment = make_comment(id, parent_id);
            comment.children = children;
            comment
        }
        // Structure of mock post + comments looks like
        // Post: 1
        //       |
        //       |- 2 - 4
        //       |  |
        //       |  | - 5
        //       |
        //       |- 3 - 6
        //       |  |
        //       |  | - 7
        let mut data_store = DataStore::init();
        let mut post = make_post(1);
        post.children = vec![2, 3];
        // id, parent id
        let comments: Vec<Comment> = vec![
            make_comment_with_children(2, 1, vec![4, 5]),
            make_comment_with_children(3, 1, vec![6, 7]),
            make_comment_with_children(4, 2, vec![]),
            make_comment_with_children(5, 2, vec![]),
            make_comment_with_children(6, 3, vec![]),
            make_comment_with_children(7, 3, vec![]),
        ];

        data_store.hydrate_posts(vec![post]);
        data_store.hydrate_comments(comments);
        // Collecting the iterator to assert the length
        let post_comments: Vec<u32> = data_store.get_post_comments(&1).collect();
        assert_eq!(post_comments.len(), 6);

        let expected_ids = [2, 4, 5, 3, 6, 7];
        for (i, comment_id) in post_comments.iter().enumerate() {
            assert_eq!(expected_ids[i], *comment_id);
        }
    }
}
