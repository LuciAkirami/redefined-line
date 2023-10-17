#![allow(unused_imports, unreachable_code, unused_must_use)]
use crossterm::{
    cursor::{
        position, MoveLeft, MoveRight, MoveToColumn, MoveToNextLine, RestorePosition, SavePosition,
    },
    event::{self, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    event::{read, Event, ModifierKeyCode},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{self, Clear, ClearType, ScrollDown, ScrollUp},
    ExecutableCommand, QueueableCommand,
};
use std::collections::VecDeque;
use std::{
    error::Error,
    io::{stdout, Result, Stdout, Write},
};
use unicode_segmentation::UnicodeSegmentation;
pub mod linebuffer;
use linebuffer::LineBuffer;

fn print_message(stdout: &mut Stdout, msg: &str) -> Result<()> {
    stdout
        //printing \n moves to next line, thus scrolling down, the terminal
        .queue(Print("\n"))?
        .queue(MoveToColumn(0))?
        .queue(Print(&msg))?
        .queue(Print("\n"))?
        .queue(MoveToColumn(0))?;
    //.queue(ScrollDown(1))?;
    stdout.flush()?;
    Ok(())
}

fn buffer_repaint(
    stdout: &mut Stdout,
    buffer: &LineBuffer,
    idx: usize,
    prompt_offset: u16,
) -> Result<()> {
    let raw_buffer = buffer.get_buffer();
    stdout
        .queue(MoveToColumn(prompt_offset))?
        .queue(Print(&raw_buffer[0..idx]))?
        .queue(SavePosition)?
        .queue(Print(&raw_buffer[idx..]))?
        .queue(Clear(ClearType::UntilNewLine))?
        .queue(RestorePosition)?;

    Ok(())
}

const HISTORY_SIZE: usize = 100;
fn main() -> std::io::Result<()> {
    // the below is so that, we don't need to specify () everytime, so we create an Stdout type
    let mut stdout = stdout();
    let mut buffer = LineBuffer::new();
    let mut history = VecDeque::with_capacity(HISTORY_SIZE);
    let mut history_cursor = -1i64;
    let mut has_history = false;

    terminal::enable_raw_mode();
    'repl: loop {
        stdout
            .execute(SetForegroundColor(Color::Magenta))?
            .execute(Print("> "))?
            .execute(ResetColor)?;
        // get the current position of the cursor, which is "2", as at 0 position there is "<" and 1st pos there is " "
        let (prompt_offset, _) = position().unwrap();
        // assign the current start position to the curser pos
        'input: loop {
            // read() lets us read the input
            match read()? {
                // matching on keypress
                Event::Key(KeyEvent {
                    code,
                    modifiers,
                    kind,
                    state: _,
                }) => {
                    // matching the kind, that is key press and key release
                    match kind {
                        // matching on key press
                        KeyEventKind::Press => {
                            // matching on code, that is if the Key is a character / enter / backspace/ Tab etc
                            match code {
                                KeyCode::Char(c) if modifiers == KeyModifiers::CONTROL => {
                                    // if pressing "Ctrl + D", exit
                                    if c == 'd' {
                                        stdout.queue(MoveToNextLine(1))?.queue(Print("\nexit"))?;
                                        break 'repl;
                                    }
                                }
                                // if the code if of type Char, then append to buffer
                                KeyCode::Char(c) => {
                                    // calc insersetion_point, so to insert char at middle
                                    let insertion_point = buffer.get_insertion_pos();
                                    // add the new char to the buffer
                                    buffer.insert_char(insertion_point, c);
                                    // empty buffer or inserting at the end, do this
                                    // if insertion_point == buffer.get_buffer_len() {
                                    //     stdout.queue(Print(c))?;
                                    // }
                                    // // if inserting somewhere in betweem do this
                                    // else {
                                    //     buffer_repaint(&mut stdout, &buffer, idx, prompt_offset)
                                    //     // stdout
                                    //     //     // print the newly inserted character
                                    //     //     .queue(Print(c))?
                                    //     //     // then print all the characters that appear after that
                                    //     //     .queue(Print(buffer.slice_buffer(insertion_point)))?
                                    //     //     // move the cursor to the end of the word/sentence
                                    //     //     .queue(MoveToColumn(
                                    //     //         (buffer.get_insertion_pos() as u16)
                                    //     //             + prompt_offset
                                    //     //             + 1,
                                    //     //     ))?;
                                    // }
                                    buffer.inc_insertion_pos();

                                    buffer_repaint(
                                        &mut stdout,
                                        &buffer,
                                        buffer.get_insertion_pos(),
                                        prompt_offset,
                                    )?;
                                    stdout.flush()?;
                                }
                                // if the code is of type backspace, then clear it
                                KeyCode::Backspace => {
                                    let insertion_point = buffer.get_insertion_pos();
                                    // check if the buffer is empty
                                    if insertion_point == buffer.get_buffer_len() as usize {
                                        if !buffer.is_buffer_empty() {
                                            // decrement the cursor position when deleting a char
                                            buffer.dec_insertion_pos();
                                            // remove the last character when backspace pressed
                                            buffer.pop();
                                            buffer_repaint(
                                                &mut stdout,
                                                &buffer,
                                                buffer.get_insertion_pos(),
                                                prompt_offset,
                                            )?;
                                            // stdout
                                            //     .queue(MoveLeft(1))?
                                            //     .queue(Print(" "))?
                                            //     .queue(MoveLeft(1))?;
                                            stdout.flush()?;
                                        }
                                    } else if insertion_point < buffer.get_buffer_len()
                                        && !buffer.is_buffer_empty()
                                    {
                                        // remember cursor pos is calculated from the start "> "
                                        // | => cursor
                                        // eg: helylo -> > hely|lo -> cursor pos = 6, removal_point = 4
                                        // buffer = hello
                                        buffer.dec_insertion_pos();
                                        let insertion_point = buffer.get_insertion_pos();
                                        buffer.remove_char(insertion_point);
                                        buffer_repaint(
                                            &mut stdout,
                                            &buffer,
                                            insertion_point,
                                            prompt_offset,
                                        );
                                        // stdout
                                        //     // in terminal: hel|ylo
                                        //     .queue(MoveLeft(1))?
                                        //     // then print all the characters that appear after that
                                        //     // hello|o
                                        //     .queue(Print(buffer.slice_buffer(insertion_point)))?
                                        //     // then to remove extra char at end, print space " "
                                        //     // hello |
                                        //     .queue(Print(" "))?
                                        //     // hello|
                                        //     // move the cursor to the position "cursor_pos-1"
                                        //     .queue(MoveToColumn(
                                        //         insertion_point as u16 + prompt_offset,
                                        //     ))?;
                                        stdout.flush()?;
                                        // hel|lo -> cursor = 5
                                    }
                                }
                                // if the code is of type backspace, then clear it
                                KeyCode::Delete => {
                                    // remember cursor pos is calculated from the start "> "
                                    // | => cursor
                                    // eg: helylo -> > hel|ylo -> cursor pos = 5, removal_point = 3
                                    // check if the buffer is empty

                                    let insertion_point = buffer.get_insertion_pos();
                                    if insertion_point < buffer.get_buffer_len() {
                                        if !buffer.is_buffer_empty() {
                                            // buffer = hello
                                            buffer.remove_char(insertion_point);
                                            buffer_repaint(
                                                &mut stdout,
                                                &buffer,
                                                insertion_point,
                                                prompt_offset,
                                            );
                                            // stdout
                                            //     // then print all the characters that appear after that
                                            //     // in terminal: hel|lo
                                            //     .queue(Print(buffer.slice_buffer(insertion_point)))?
                                            //     // then to remove extra char at end, print space " "
                                            //     // hello|
                                            //     .queue(Print(" "))?
                                            //     // hell|o
                                            //     // move the cursor to curser_pos, that is cursor_pos=5
                                            //     .queue(MoveToColumn(
                                            //         insertion_point as u16 + prompt_offset,
                                            //     ))?;
                                            stdout.flush()?;
                                        }
                                    }
                                }
                                KeyCode::Left => {
                                    // the cursor position can move only till before the "<"
                                    if buffer.get_insertion_pos() > 0 {
                                        if modifiers == KeyModifiers::ALT {
                                            let new_insertion_point = buffer.move_word_left();
                                            stdout.queue(MoveToColumn(
                                                new_insertion_point as u16 + prompt_offset,
                                            ))?;
                                        } else {
                                            buffer.dec_insertion_pos();
                                            let insertion_point = buffer.get_insertion_pos();
                                            //let idx = buffer.get_grapheme_idx_left();
                                            buffer_repaint(
                                                &mut stdout,
                                                &buffer,
                                                insertion_point,
                                                prompt_offset,
                                            );
                                            //let raw_buffer = buffer.get_buffer();

                                            // stdout
                                            //     .queue(MoveToColumn(prompt_offset))?
                                            //     .queue(Print(&raw_buffer[0..idx]))?
                                            //     .queue(SavePosition)?
                                            //     .queue(Print(&raw_buffer[idx..]))?
                                            //     .queue(RestorePosition)?;

                                            // stdout.queue(MoveLeft(1))?;
                                            // decrement the cursor position when moving towards left
                                        }
                                        stdout.flush()?;
                                    }
                                }

                                KeyCode::Right => {
                                    if buffer.get_insertion_pos() < buffer.get_buffer_len() {
                                        if modifiers == KeyModifiers::ALT {
                                            let new_insertion_point = buffer.move_word_right();
                                            if new_insertion_point > 0 {
                                                stdout.queue(MoveToColumn(
                                                    new_insertion_point as u16 + prompt_offset,
                                                ))?;
                                            } else {
                                                stdout.queue(MoveToColumn(prompt_offset))?;
                                            }
                                            //stdout.queue(MoveToColumn(new_insertion_point as u16 + prompt_offset))?;
                                        }
                                        // the cursoir position can move right until it reaches the last char
                                        else {
                                            buffer.inc_insertion_pos();
                                            let insertion_point = buffer.get_insertion_pos();
                                            //let idx = buffer.get_grapheme_idx_left();
                                            buffer_repaint(
                                                &mut stdout,
                                                &buffer,
                                                insertion_point,
                                                prompt_offset,
                                            );
                                            // let idx = buffer.get_grapheme_idx_right();
                                            // let raw_buffer = buffer.get_buffer();

                                            // stdout
                                            //     .queue(MoveToColumn(prompt_offset))?
                                            //     .queue(Print(&raw_buffer[0..idx]))?
                                            //     .queue(SavePosition)?
                                            //     .queue(Print(&raw_buffer[idx..]))?
                                            //     .queue(RestorePosition)?;
                                            // // stdout.queue(MoveRight(1))?;
                                            // // increment the cursor position when moving towards right
                                            // buffer.inc_insertion_pos();
                                        }

                                        stdout.flush()?;
                                    }
                                }
                                KeyCode::Enter => {
                                    //println!("\nOur Buffer: {}",buffer);
                                    if buffer.get_buffer() == "exit" {
                                        break 'repl;
                                    } else {
                                        if history.len() + 1 == HISTORY_SIZE {
                                            // History is "full", so we delete the oldest entry first,
                                            // before adding a new one.
                                            history.pop_back();
                                        }
                                        history.push_front(String::from(buffer.get_buffer()));
                                        has_history = true;
                                        // reset the history cursor - we want to start at the bottom of the
                                        // history again.
                                        history_cursor = -1;
                                        print_message(
                                            &mut stdout,
                                            &format!("Our buffer: {}", buffer.get_buffer()),
                                        )?;
                                        buffer.clear_buffer();
                                        buffer.set_insertion_pos(0);
                                        break 'input;
                                    }
                                }

                                KeyCode::Up => {
                                    if has_history && history_cursor < (history.len() as i64 - 1) {
                                        history_cursor += 1;
                                        let history_word =
                                            history.get(history_cursor as usize).unwrap().clone();
                                        buffer.set_buffer(history_word);
                                        buffer.move_to_end();
                                        buffer_repaint(
                                            &mut stdout,
                                            &buffer,
                                            buffer.get_insertion_pos(),
                                            prompt_offset,
                                        )?;
                                        stdout.flush()?;
                                    }
                                }

                                KeyCode::Down => {
                                    if has_history && history_cursor >= 0 {
                                        history_cursor -= 1;
                                        // if clicked down after reaching bottom of history, we start with
                                        // empty buffer
                                        let history_word = if history_cursor < 0 {
                                            String::new()
                                        } else {
                                            history.get(history_cursor as usize).unwrap().clone()
                                        };

                                        buffer.set_buffer(history_word);
                                        buffer.move_to_end();
                                        buffer_repaint(
                                            &mut stdout,
                                            &buffer,
                                            buffer.get_insertion_pos(),
                                            prompt_offset,
                                        )?;
                                        stdout.flush()?;
                                    }
                                }

                                KeyCode::Home => {
                                    stdout.queue(MoveToColumn(prompt_offset));
                                    stdout.flush();
                                    buffer.set_insertion_pos(0)
                                }
                                KeyCode::End => {
                                    let buffer_len = buffer.get_buffer_len();
                                    buffer_repaint(
                                        &mut stdout,
                                        &buffer,
                                        buffer_len,
                                        prompt_offset,
                                    )?;
                                    stdout.flush();
                                    buffer.set_insertion_pos(buffer_len)
                                }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                }
                Event::Mouse(event) => {
                    println!("{:?}", event)
                }
                Event::Resize(width, height) => {
                    print_message(
                        &mut stdout,
                        &format!("width: {}, height: {}", width, height),
                    );
                    break 'input;
                }
                Event::Paste(buff) => {
                    print_message(&mut stdout, &format!("{:?}", buff));
                }
                _ => {}
            }
        }
    }

    terminal::disable_raw_mode();
    //println!("\x1b[0m");

    Ok(())
}
