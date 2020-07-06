mod constants;
mod hn_client;
mod stores;
mod ui;

use std::io::{self, Write};

use crossterm::{queue, style::Print, terminal::size};

use constants::{PAGE_SIZE, POST_ROW_SIZE};
use stores::data::{init_store};
use stores::view::{init_view_state, Page, StoryListType, ScrollDirection};


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();
    ui::initialize_screen(&mut stdout)?;
    let mut view_state = init_view_state();
    let mut data_store = init_store();

    // TODO - move this into store fns
    let post_ids = hn_client::get_post_ids(StoryListType::Top).await?;
    let paginated_post_ids = &post_ids[0..0 + PAGE_SIZE as usize];
    let posts = hn_client::get_stories(paginated_post_ids).await?;

    for post in posts {
        data_store.top_post_ids.push(post.id);
        data_store.posts.insert(post.id, post);
    }

    loop {
        ui::clear_screen(&mut stdout)?;
        let (columns, rows) = size()?;
        match view_state.page {
            Page::PostList {
                cursor_index,
                ..
            } => {
                // Calculate number of posts that can fit in the terminal
                // Remove from total rows - end, etc - 1 row for commands
                // Add one to handle render overflows
                let number_of_posts = (rows - 1) / POST_ROW_SIZE as u16 + 1;
                for (i, post_id) in data_store
                    .top_post_ids
                    .iter()
                    .skip(view_state.scroll_offset as usize)
                    .take(number_of_posts as usize)
                    .enumerate()
                {
                    let n = i + view_state.scroll_offset as usize;
                    // TODO create a render page post list fn
                    let post = data_store.posts.get(post_id).unwrap();
                    let cursor_text = if cursor_index as usize == n {
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
            _ => {
                // TO IMPLEMENT
            }
        };
    }
    ui::teardown_screen(&mut stdout)?;

    Ok(())
}
