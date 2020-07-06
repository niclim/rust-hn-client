mod hn_client;
mod store;
mod ui;

use std::io::{self, Write};

use crossterm::{queue, style::Print, terminal::size};

use store::{init_store, ViewState, Page};
use ui::POST_ROW_SIZE;

const PAGE_SIZE: u8 = 20;



#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();
    ui::initialize_screen(&mut stdout)?;
    let mut view_state = ViewState {
        page: Page::PostList {
            offset: 0,
            cursor_index: 0,
        },
        scroll_offset: 0,
    };
    let mut data_store = init_store();

    // For now, load this on app initialize - we'll want to move this into some action trigger
    // to render loading state
    let posts =
        hn_client::get_stories(hn_client::StoryListType::Top, 0, PAGE_SIZE as usize).await?;
    for post in posts {
        data_store.top_post_ids.push(post.id);
        data_store.posts.insert(post.id, post);
    }
    // if top_posts.len() > 0 {
    //     let id = top_posts[1];
    //     let post = post_hash.get(&id).unwrap();
    //     let comments = hn_client::get_comments(&post.children).await?;

    //     for comment in comments {
    //         comment_hash.insert(comment.id, comment);
    //     }
    // }

    loop {
        ui::clear_screen(&mut stdout)?;
        let (columns, rows) = size()?;
        match view_state.page {
            Page::PostList {
                offset,
                cursor_index,
            } => {
                // Calculate number of posts that can fit in the terminal
                // Remove from total rows - end, etc - 1 row for commands
                // Add one to handle render overflows
                let number_of_posts = (rows - 1) / POST_ROW_SIZE as u16 + 1;
                for (i, post_id) in data_store.top_post_ids
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
            ui::UserAction::Up => scroll(&mut view_state, rows, Direction::Up),
            ui::UserAction::Down => scroll(&mut view_state, rows, Direction::Down),
            _ => {
                // TO IMPLEMENT
            }
        };
    }
    ui::teardown_screen(&mut stdout)?;

    Ok(())
}

enum Direction {
    Up,
    Down,
}

fn scroll(view_state: &mut ViewState, rows: u16, direction: Direction) {
    match view_state.page {
        Page::PostList {
            offset,
            cursor_index,
        } => {
            // Calculate number of posts that can be shown without overflow / crop
            let number_of_posts = (rows - 1) / POST_ROW_SIZE as u16;
            // adjust cursor position
            let new_cursor: u32 = match direction {
                Direction::Up => {
                    if cursor_index == 0 {
                        0
                    } else {
                        cursor_index - 1
                    }
                }
                Direction::Down => {
                    if cursor_index == (PAGE_SIZE - 1) as u32 {
                        cursor_index
                    } else {
                        cursor_index + 1
                    }
                }
            };
            let scroll_offset = match direction {
                Direction::Up => {
                    if view_state.scroll_offset as u32 > new_cursor {
                        view_state.scroll_offset - 1
                    } else {
                        view_state.scroll_offset
                    }
                }
                Direction::Down => {
                    if new_cursor > (view_state.scroll_offset + number_of_posts) as u32 {
                        view_state.scroll_offset + 1
                    } else {
                        view_state.scroll_offset
                    }
                }
            };

            view_state.page = Page::PostList {
                offset: offset,
                cursor_index: new_cursor,
            };
            view_state.scroll_offset = scroll_offset;
        }
        Page::PostDetails { post, cursor_index } => {
            // TODO - implement
        }
    };
}
