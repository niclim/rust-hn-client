use crate::stores::data::Post;
use crate::constants::LEFT_OFFSET;
use std::io::Write;

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, MouseEvent},
    execute, queue,
    style::{self, Print},
    terminal::{self, ClearType},
    Result as CrossTermResult,
};

pub enum UserAction {
    Up,
    Down,
    Enter,
    // Refresh,
    Rerender,
    Quit,
}

pub fn clear_screen<W>(w: &mut W) -> CrossTermResult<()>
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

pub fn initialize_screen<W>(w: &mut W) -> CrossTermResult<()>
where
    W: Write,
{
    execute!(w, terminal::EnterAlternateScreen)?;
    terminal::enable_raw_mode()
}

pub fn teardown_screen<W>(w: &mut W) -> CrossTermResult<()>
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

pub fn get_user_action() -> CrossTermResult<UserAction> {
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
            return Ok(UserAction::Rerender);
        }
    }
}

pub fn print_post<W>(w: &mut W, number: usize, columns: u16, post: &Post) -> CrossTermResult<()>
where
    W: Write,
{
    // Posts will take up exactly 3 rows - things will be cropped otherwise
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
