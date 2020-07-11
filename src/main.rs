mod constants;
mod hn_client;
mod stores;
mod ui;

use std::future;
use std::io::{self, Write};

use crossterm::{queue, style::Print, terminal::size};

use constants::{PAGE_SIZE, POST_ROW_SIZE};
use stores::data::DataStore;
use stores::view::{Page, ScrollDirection, StoryListType, ViewState};

enum AsyncAction {
    Noop,
    FetchPosts { filter: StoryListType, offset: u32 },
    FetchComments { comment_ids: Vec<u32> },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();
    ui::initialize_screen(&mut stdout)?;
    let mut view_state = ViewState::init();
    let mut data_store = DataStore::init();
    let mut async_action: AsyncAction = AsyncAction::FetchPosts {
        filter: StoryListType::Top,
        offset: 0,
    };

    loop {
        match &async_action {
            AsyncAction::FetchPosts { filter, offset } => {
                // TODO - add in loading screen state

                // Only load posts if this has not been added to the store
                if !data_store.has_post_ids(filter) {
                    let post_ids = hn_client::get_post_ids(filter).await?;
                    data_store.hydrate_post_ids(filter, post_ids);
                }

                let post_ids = data_store.get_post_ids(filter);
                let start = *offset as usize;
                let end = (*offset + PAGE_SIZE as u32) as usize;
                let paginated_post_ids = &post_ids[start..end];
                // TODO - filter posts that are already loaded

                let posts = hn_client::get_stories(paginated_post_ids).await?;

                data_store.hydrate_posts(posts);
            }
            AsyncAction::FetchComments { comment_ids } => {
                // check if fetch otherwise fetch
                let comments = hn_client::get_comments(comment_ids).await?;
                // TODO hydrate store
            }
            AsyncAction::Noop => {}
        };
        async_action = AsyncAction::Noop;

        ui::clear_screen(&mut stdout)?;

        let (columns, rows) = size()?;
        match &view_state.page {
            Page::PostList {
                cursor_index,
                filter,
                ..
            } => {
                // Calculate number of posts that can fit in the terminal
                // Remove from total rows - end, etc - 1 row for commands
                // Add one to handle render overflows
                let number_of_posts = (rows - 1) / POST_ROW_SIZE as u16 + 1;
                for (i, post_id) in data_store
                    .get_post_ids(filter)
                    .iter()
                    .skip(view_state.scroll_offset as usize)
                    .take(number_of_posts as usize)
                    .enumerate()
                {
                    let n = i + view_state.scroll_offset as usize;
                    // TODO create a render page post list fn
                    // TODO handle error case here
                    let post = data_store.get_post(post_id).unwrap();
                    let cursor_text = if *cursor_index as usize == n {
                        "➜  "
                    } else {
                        "   "
                    };
                    queue!(stdout, Print(cursor_text))?;
                    ui::print_post(&mut stdout, n, columns, post)?;
                }
            }
            Page::PostDetails { post, cursor_index } => {
                // TODO - implement
            }
        };
        stdout.flush()?;

        match ui::get_user_action()? {
            ui::UserAction::Quit => break,
            ui::UserAction::Up => view_state.scroll(rows, ScrollDirection::Up),
            ui::UserAction::Down => view_state.scroll(rows, ScrollDirection::Down),
            // ui::UserAction::Enter => {async_action = AsyncAction::Noop;}
            _ => {
                // TO IMPLEMENT
            }
        };
    }
    ui::teardown_screen(&mut stdout)?;

    Ok(())
}
