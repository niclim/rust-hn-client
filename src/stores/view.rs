use crate::constants::{PAGE_SIZE, POST_ROW_SIZE};

pub enum StoryListType {
  New,
  Best,
  Top,
}

pub struct ViewState {
  pub page: Page,
  pub scroll_offset: u16,
}

pub enum Page {
  PostList {
    offset: u32,
    cursor_index: u32,
    sort: StoryListType,
  },
  PostDetails {
    post: u32,
    cursor_index: u32,
  },
}

pub fn init_view_state() -> ViewState {
  ViewState {
    page: Page::PostList {
      offset: 0,
      cursor_index: 0,
      sort: StoryListType::Top,
    },
    scroll_offset: 0,
  }
}

pub enum ScrollDirection {
  Up,
  Down,
}

impl ViewState {
  pub fn scroll(&mut self, rows: u16, direction: ScrollDirection) {
    match &self.page {
      Page::PostList {
        cursor_index,
        offset,
        sort,
      } => {
        // Calculate number of posts that can be shown without overflow / crop
        let number_of_posts = (rows - 1) / POST_ROW_SIZE as u16;
        // adjust cursor position
        let new_cursor: u32 = match direction {
          ScrollDirection::Up => {
            if cursor_index == &0 {
              0
            } else {
              cursor_index - 1
            }
          }
          ScrollDirection::Down => {
            if cursor_index == &((PAGE_SIZE - 1) as u32) {
              *cursor_index
            } else {
              cursor_index + 1
            }
          }
        };
        let scroll_offset = match direction {
          ScrollDirection::Up => {
            if self.scroll_offset as u32 > new_cursor {
              self.scroll_offset - 1
            } else {
              self.scroll_offset
            }
          }
          ScrollDirection::Down => {
            if new_cursor > (self.scroll_offset + number_of_posts) as u32 {
              self.scroll_offset + 1
            } else {
              self.scroll_offset
            }
          }
        };

        self.page = Page::PostList {
          offset: *offset,
          cursor_index: new_cursor,
          sort: match sort {
            StoryListType::Top => StoryListType::Top,
            StoryListType::Best => StoryListType::Best,
            StoryListType::New => StoryListType::New,
          }
        };
        self.scroll_offset = scroll_offset;
      }
      Page::PostDetails { post, cursor_index } => {
        // TODO - implement
      }
    };
  }
}
