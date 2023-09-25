use color_eyre::eyre::Result;
use tokio::sync::mpsc;

use crate::{
  command::Command,
  components::{app::App, Component},
  config::Config,
  tui,
};

pub struct Runner {
  pub config: Config,
  pub tick_rate: f64,
  pub frame_rate: f64,
  pub components: Vec<Box<dyn Component>>,
  pub should_quit: bool,
  pub should_suspend: bool,
}

impl Runner {
  pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
    let app = App::new();
    let config = Config::new()?;
    let app = app.keybindings(config.keybindings.clone());
    Ok(Self {
      tick_rate,
      frame_rate,
      components: vec![Box::new(app)],
      should_quit: false,
      should_suspend: false,
      config,
    })
  }

  pub async fn run(&mut self) -> Result<()> {
    let (command_tx, mut command_rx) = mpsc::unbounded_channel();

    let mut tui = tui::Tui::new()?;
    tui.tick_rate(self.tick_rate);
    tui.frame_rate(self.frame_rate);
    tui.enter()?;

    for component in self.components.iter_mut() {
      component.register_command_handler(command_tx.clone())?;
    }

    for component in self.components.iter_mut() {
      component.init()?;
    }

    loop {
      if let Some(e) = tui.next().await {
        match e {
          tui::Event::Quit => command_tx.send(Command::Quit)?,
          tui::Event::Tick => command_tx.send(Command::Tick)?,
          tui::Event::Render => command_tx.send(Command::Render)?,
          tui::Event::Resize(x, y) => command_tx.send(Command::Resize(x, y))?,
          e => {
            for component in self.components.iter_mut() {
              if let Some(command) = component.handle_events(Some(e.clone()))? {
                command_tx.send(command)?;
              }
            }
          },
        }
      }

      while let Ok(command) = command_rx.try_recv() {
        if command != Command::Tick && command != Command::Render {
          log::debug!("{command:?}");
        }
        match command {
          Command::Quit => self.should_quit = true,
          Command::Suspend => self.should_suspend = true,
          Command::Resume => self.should_suspend = false,
          Command::Render => {
            tui.draw(|f| {
              for component in self.components.iter_mut() {
                let r = component.draw(f, f.size());
                if let Err(e) = r {
                  command_tx.send(Command::Error(format!("Failed to draw: {:?}", e))).unwrap();
                }
              }
            })?;
          },
          _ => {},
        }
        for component in self.components.iter_mut() {
          if let Some(command) = component.update(command.clone())? {
            command_tx.send(command)?
          };
        }
      }
      if self.should_suspend {
        tui.suspend()?;
        command_tx.send(Command::Resume)?;
        tui = tui::Tui::new()?;
        tui.tick_rate(self.tick_rate);
        tui.frame_rate(self.frame_rate);
        tui.enter()?;
      } else if self.should_quit {
        tui.stop()?;
        break;
      }
    }
    tui.exit()?;
    Ok(())
  }
}
