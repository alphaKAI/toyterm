use glium::{glutin, Display};
use glutin::{
    event::{Event, WindowEvent},
    event_loop::ControlFlow,
};

use crate::window::TerminalWindow;

const PREFIX_KEY: char = '\x01'; // Ctrl + A

pub struct Multiplexer {
    display: Display,
    select: usize,
    wins: Vec<Option<TerminalWindow>>,
    consume: bool,
}

impl Multiplexer {
    pub fn new(display: Display) -> Self {
        Multiplexer {
            display,
            select: 0,
            wins: Vec::new(),
            consume: false,
        }
    }

    pub fn allocate_new_window(&mut self) -> usize {
        log::info!("new terminal window added");
        let new = TerminalWindow::new(self.display.clone());
        let num = self.wins.len();
        self.wins.push(Some(new));
        num
    }

    pub fn on_event(&mut self, event: &Event<()>, control_flow: &mut ControlFlow) {
        if self.wins.is_empty() {
            *control_flow = ControlFlow::Exit;
            return;
        }

        match &event {
            Event::WindowEvent {
                event: win_event, ..
            } => match win_event {
                WindowEvent::CloseRequested => {
                    *control_flow = ControlFlow::Exit;
                    return;
                }

                // Broadcast
                WindowEvent::ModifiersChanged(..) | WindowEvent::Resized(..) => {
                    for win in self.wins.iter_mut() {
                        let mut cf = ControlFlow::default();
                        if let Some(win) = win {
                            win.on_event(event, &mut cf);
                        }
                        if cf == ControlFlow::Exit {
                            *win = None;
                        }
                    }
                    self.wins.retain(|win| win.is_some());

                    if self.select >= self.wins.len() {
                        self.select = 0;
                    }

                    if self.wins.is_empty() {
                        *control_flow = ControlFlow::Exit;
                    }
                    return;
                }

                WindowEvent::ReceivedCharacter(PREFIX_KEY) if !self.consume => {
                    self.consume = true;
                    return;
                }

                WindowEvent::ReceivedCharacter(PREFIX_KEY) if self.consume => {
                    self.consume = false;
                }

                WindowEvent::ReceivedCharacter('\x1B') if self.consume => {
                    // Esc
                    self.consume = false;
                    return;
                }

                // Create a new window
                WindowEvent::ReceivedCharacter('c') if self.consume => {
                    self.select = self.allocate_new_window();
                    self.consume = false;
                    return;
                }

                // Next
                WindowEvent::ReceivedCharacter('n') if self.consume => {
                    log::debug!("next window");
                    self.select += 1;
                    self.select %= self.wins.len();
                    self.consume = false;
                    return;
                }
                // Prev
                WindowEvent::ReceivedCharacter('p') if self.consume => {
                    log::debug!("prev window");
                    self.select = self.wins.len() + self.select - 1;
                    self.select %= self.wins.len();
                    self.consume = false;
                    return;
                }

                // Just ignore other characters
                WindowEvent::ReceivedCharacter(_) if self.consume => {
                    self.consume = false;
                    return;
                }

                _ => {}
            },
            _ => {}
        }

        // Forward to the selected window
        let mut cf = ControlFlow::default();

        self.wins[self.select]
            .as_mut()
            .unwrap()
            .on_event(event, &mut cf);

        if cf == ControlFlow::Exit {
            // remove selected window
            self.wins.remove(self.select);

            if self.select == self.wins.len() {
                self.select = 0;
            }

            if self.wins.is_empty() {
                *control_flow = ControlFlow::Exit;
            }
        }
    }
}
