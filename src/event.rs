use std::time::Duration;

use crossterm::event::{Event as CrosstermEvent, KeyEvent};
use futures::{FutureExt, StreamExt};
use tokio::sync::mpsc;
use crate::app::AppState;
use crate::ro_cell::RoCell;

static TX: RoCell<mpsc::UnboundedSender<Event>> = RoCell::new();

/// Terminal events.
#[derive(Clone, Debug)]
pub enum Event {
    /// Terminal tick.
    Tick,
    /// Key press.
    Key(KeyEvent),
    /// Mouse click/scroll.
    // Mouse(MouseEvent),
    /// Terminal resize.
    Resize(u16, u16),
    /// render current pdf page
    RenderPdf,
    /// Load next pdf page
    LoadingNext,
    /// load pdf first
    LoadingFirst(u32),
    /// change state
    ChangeState(AppState),
}

/// Terminal event handler.
#[allow(dead_code)]
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    sender: mpsc::UnboundedSender<Event>,
    /// Event receiver channel.
    receiver: mpsc::UnboundedReceiver<Event>,
    /// Event handler thread.
    handler: tokio::task::JoinHandle<()>,
}

impl Event {
    pub fn init(tx: mpsc::UnboundedSender<Event>) {
        TX.init(tx);
    }

    pub fn emit(self) {
        TX.send(self).ok();
    }
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`].
    pub fn new(tick_rate: u64) -> Self {
        let tick_rate = Duration::from_millis(tick_rate);
        let (sender, receiver) = mpsc::unbounded_channel();
        let _sender = sender.clone();
        let handler = tokio::spawn(async move {
            let mut reader = crossterm::event::EventStream::new();
            let mut tick = tokio::time::interval(tick_rate);
            loop {
                let tick_delay = tick.tick();
                let crossterm_event = reader.next().fuse();
                tokio::select! {
                  _ = _sender.closed() => {
                    break;
                  }
                  _ = tick_delay => {
                    _sender.send(Event::Tick).unwrap();
                  }
                  Some(Ok(evt)) = crossterm_event => {
                    match evt {
                      CrosstermEvent::Key(key) => {
                        if key.kind == crossterm::event::KeyEventKind::Press {
                          _sender.send(Event::Key(key)).unwrap();
                        }
                      },
                      CrosstermEvent::Mouse(_mouse) => {
                        // _sender.send(Event::Mouse(mouse)).unwrap();
                      },
                      CrosstermEvent::Resize(x, y) => {
                        _sender.send(Event::Resize(x, y)).unwrap();
                      },
                      CrosstermEvent::FocusLost => {
                      },
                      CrosstermEvent::FocusGained => {
                      },
                      CrosstermEvent::Paste(_) => {
                      },
                    }
                  }
                }
                ;
            }
        });
        Event::init(sender.clone());
        Self {
            sender,
            receiver,
            handler,
        }
    }

    /// Receive the next event from the handler thread.
    ///
    /// This function will always block the current thread if
    /// there is no data available and it's possible for more data to be sent.
    pub async fn next(&mut self) -> anyhow::Result<Event> {
        let event = self.receiver
            .recv()
            .await
            .unwrap();
        Ok(event)
    }

    pub fn get_sender(&self) -> mpsc::UnboundedSender<Event> {
        self.sender.clone()
    }
}


#[macro_export]
macro_rules! emit {
    (RenderPdf) => {
        $crate::event::Event::RenderPdf.emit()
    };
    (LoadingFirst($page_id: expr)) => {
        $crate::event::Event::LoadingFirst($page_id).emit()
    };
    (LoadingNext) => {
        $crate::event::Event::LoadingNext.emit()
    };
    (ChangeState($state: expr)) => {
        $crate::event::Event::ChangeState($state).emit()
    };
}