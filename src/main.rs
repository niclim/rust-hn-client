use std::collections::HashMap;
use std::io::{self, Write};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute, queue,
    style::{
        self, Print,
    },
    terminal::{self, ClearType},
    Result as CrossTermResult,
};

mod hn_client;
use hn_client::{Comment, Post};

fn read_char() -> CrossTermResult<char> {
    loop {
        if let Ok(Event::Key(KeyEvent {
            code: KeyCode::Char(c),
            ..
        })) = event::read()
        {
            return Ok(c);
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

// TODO move this into a trait
fn print_post<W>(w: &mut W, number: usize, post: &Post) -> CrossTermResult<()>
where
    W: Write,
{
    let post_string = format!(
        r#"{number} - {post_title} - {post_author}
{time} - {descendants} comments"#,
        number = number + 1,
        post_title = post.title,
        post_author = post.by,
        time = post.time,
        descendants = post.descendants
    );
    for line in post_string.split("\n") {
        queue!(w, Print(line), cursor::MoveToNextLine(1));
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout = io::stdout();
    initialize_screen(&mut stdout)?;
    // Initialize (mutable) store here
    // TODO create a store file + interactions to modify store
    let mut top_posts: Vec<u32> = Vec::new();
    let mut post_hash: HashMap<u32, Post> = HashMap::new();
    // let mut comment_hash: HashMap<u32, Comment> = HashMap::new();
    // TODO add view state (i.e. post list vs posts, cursor position, page offset (scroll position), page state (pagination, current post))

    // For now, load this on app initialize - we'll want to move this into some action trigger
    // to render loading state
    let posts = hn_client::get_stories(hn_client::StoryListType::Top, 0, 20).await?;
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
        // TODO implement render post list page for now
        for (i, post_id) in top_posts.iter().enumerate() {
            let post = post_hash.get(post_id).unwrap();
            print_post(&mut stdout, i, post)?;
        }
        stdout.flush()?;

        // TODO - impl up down + ctrl+c
        match read_char()? {
            'q' => break,
            _ => {}
        };
    }
    teardown_screen(&mut stdout)?;

    Ok(())
}
