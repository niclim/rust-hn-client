use std::collections::HashMap;
use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, MouseEvent},
    execute, queue,
    style::{self, Print},
    terminal::{self, size, ClearType},
    Result as CrossTermResult,
};

mod hn_client;
use hn_client::{Comment, Post};

enum UserAction {
    Up,
    Down,
    Enter,
    Refresh,
    Quit,
}

fn get_user_action() -> CrossTermResult<UserAction> {
    loop {
        if let Ok(Event::Key(KeyEvent { code, .. })) = event::read() {
            match code {
                KeyCode::Char('q') => return Ok(UserAction::Quit),
                KeyCode::Esc => return Ok(UserAction::Quit),
                KeyCode::Up => return Ok(UserAction::Up),
                KeyCode::Down => return Ok(UserAction::Down),
                KeyCode::Enter => return Ok(UserAction::Enter),
                _ => continue,
            }
        } else if let Ok(Event::Mouse(mouse_event)) = event::read() {
            match mouse_event {
                MouseEvent::ScrollUp(_, _, _) => return Ok(UserAction::Up),
                MouseEvent::ScrollDown(_, _, _) => return Ok(UserAction::Down),
                _ => continue,
            }
        } else if let Ok(Event::Resize(_, _)) = event::read() {
            return Ok(UserAction::Refresh);
        }
    }
}

fn clear_screen<W>(w: &mut W) -> CrossTermResult<()>
where
    W: Write,
{
    queue!(
        w,
        style::ResetColor,
        terminal::Clear(ClearType::All),
        cursor::Hide,
        cursor::MoveTo(0, 0)
    )
}

fn initialize_screen<W>(w: &mut W) -> CrossTermResult<()>
where
    W: Write,
{
    execute!(w, terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()
}

fn teardown_screen<W>(w: &mut W) -> CrossTermResult<()>
where
    W: Write,
{
    execute!(
        w,
        style::ResetColor,
        cursor::Show,
        terminal::LeaveAlternateScreen
    )?;
    terminal::disable_raw_mode()
}

const LEFT_OFFSET: u16 = 3;
const POST_ROW_SIZE: u8 = 3;
const PAGE_SIZE: u8 = 20;

// TODO move this into a trait
// A post size takes up 4 rows
fn print_post<W>(w: &mut W, number: usize, columns: u16, post: &Post) -> CrossTermResult<()>
where
    W: Write,
{
    // Posts will take up exactly 4 rows - things will be cropped otherwise
    // TODO - handle size constraints - crop post title if long
    let main_line = format!(
        "{number} - {post_title}",
        number = number + 1,
        post_title = post.title,
    );
    let sub_line = format!(
        "{post_author} - {time} - {descendants} comments",
        post_author = post.by,
        time = post.time,
        descendants = post.descendants
    );
    queue!(w, Print(main_line), cursor::MoveToNextLine(1),)?;
    queue!(
        w,
        cursor::MoveRight(LEFT_OFFSET + 4),
        Print(sub_line),
        cursor::MoveToNextLine(1),
    )?;
    queue!(w, cursor::MoveRight(LEFT_OFFSET), cursor::MoveToNextLine(1))?;

    Ok(())
}

struct ViewState {
    page: Page,
    scroll_offset: u16,
}

enum Page {
    PostList { page: u32, cursor: u32 },
    PostDetails { post: u32, cursor: u32 },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();
    initialize_screen(&mut stdout)?;
    let mut view_state = ViewState {
        page: Page::PostList { page: 0, cursor: 0 },
        scroll_offset: 0,
    };
    let mut top_posts: Vec<u32> = Vec::new();
    let mut post_hash: HashMap<u32, Post> = HashMap::new();
    // let mut comment_hash: HashMap<u32, Comment> = HashMap::new();
    // TODO add view state (i.e. post list vs posts, cursor position, page offset (scroll position), page state (pagination, current post))

    // For now, load this on app initialize - we'll want to move this into some action trigger
    // to render loading state
    let posts = hn_client::get_stories(hn_client::StoryListType::Top, 0, PAGE_SIZE as usize).await?;
    for post in posts {
        top_posts.push(post.id);
        post_hash.insert(post.id, post);
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
        clear_screen(&mut stdout)?;
        let (columns, rows) = size()?;
        match view_state.page {
            Page::PostList { page, cursor } => {
                // Calculate number of posts that can fit in the terminal
                // Remove from total rows - end, etc - 1 row for commands
                // Add one to handle render overflows
                let number_of_posts = (rows - 1) / POST_ROW_SIZE as u16 + 1;
                for (i, post_id) in top_posts
                    .iter()
                    .skip(view_state.scroll_offset as usize)
                    .take(number_of_posts as usize)
                    .enumerate()
                {
                    let n = i + view_state.scroll_offset as usize;
                    // Implement offsets / calculate size
                    let post = post_hash.get(post_id).unwrap();
                    let cursor_text = if cursor as usize == n { "➜  " } else { "   " };
                    queue!(stdout, Print(cursor_text))?;
                    print_post(&mut stdout, n, columns, post)?;
                }
            }
            Page::PostDetails { post, cursor } => {
                // TODO - implement
            }
        };
        stdout.flush()?;

        match get_user_action()? {
            UserAction::Quit => break,
            UserAction::Up => scroll(&mut view_state, rows, Direction::Up),
            UserAction::Down => scroll(&mut view_state, rows, Direction::Down),
            _ => {
                // TO IMPLEMENT
            }
        };
    }
    teardown_screen(&mut stdout)?;

    Ok(())
}

enum Direction {
    Up,
    Down,
}

fn scroll(view_state: &mut ViewState, rows: u16, direction: Direction) {
    match view_state.page {
        Page::PostList { page, cursor } => {
            // Calculate number of posts that can be shown without overflow / crop
            let number_of_posts = (rows - 1) / POST_ROW_SIZE as u16;
            // adjust cursor position
            let new_cursor: u32 = match direction {
                Direction::Up => if cursor == 0 { 0 } else { cursor - 1},
                Direction::Down => if cursor == (PAGE_SIZE - 1) as u32 { cursor } else { cursor + 1},
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
                page: page,
                cursor: new_cursor,
            };
            view_state.scroll_offset = scroll_offset;
        }
        Page::PostDetails { post, cursor } => {
            // TODO - implement
        }
    };
}
