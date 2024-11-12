use std::{error::Error, fs, path::PathBuf};

mod files;
mod global;
mod handler;
mod info;
mod progress;
mod search;

#[macro_use]
extern crate log;

fn main() {
  match run() {
    Ok(()) => {
      info!("exiting");
    }
    Err(e) => {
      error!("{}", e);
      std::process::exit(1);
    }
  }
}

fn run() -> Result<(), Box<dyn Error>> {
  setup_logging();

  // TODO: Use an epoll loop instead of spawning all these threads.
  let (connection, io_threads) = lsp_server::Connection::stdio();

  let (initialize_id, initialize_params) = match connection.initialize_start() {
    Ok(it) => it,
    Err(e) => {
      if e.channel_is_disconnected() {
        io_threads.join()?;
      }
      return Err(e.into());
    }
  };
  // FIXME: Need to not have a single root.
  #[allow(deprecated)]
  let lsp_types::InitializeParams { root_uri, .. } =
    serde_json::from_value::<lsp_types::InitializeParams>(initialize_params)?;

  info!("starting LSP server in project root: {:?}", root_uri);

  let server_capabilities = info::server_capabilities();

  let initialize_result = lsp_types::InitializeResult {
    capabilities: server_capabilities,
    server_info:  Some(lsp_types::ServerInfo {
      name:    String::from("mclsp"),
      version: Some(info::version().to_string()),
    }),
  };

  let initialize_result = serde_json::to_value(initialize_result).unwrap();

  if let Err(e) = connection.initialize_finish(initialize_id, initialize_result) {
    if e.channel_is_disconnected() {
      io_threads.join()?;
    }
    return Err(e.into());
  }

  let mut global = global::GlobalState::new(connection.sender);

  match mc_gradle::extract_jar() {
    Ok(path) => {
      info!("extracted minecraft jar to: {}", path.display());
      // global.set_client_path(it);
    }
    Err(e) => {
      error!("failed to extract minecraft jar: {}", e);
    }
  }

  let workspace = crate::search::discover_workspace(&mut global.files.write());
  global.set_workspace(workspace);

  global.run(connection.receiver)?;

  Ok(())
}

fn setup_logging() {
  let dir = PathBuf::from("/home/macmv/.cache/mclsp");
  fs::create_dir_all(&dir).unwrap();

  fern::Dispatch::new()
    .format(|out, message, record| {
      out.finish(format_args!(
        "[{level}][{target}] {message}",
        // date = chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
        level = record.level(),
        target = record.target(),
        message = message,
      ))
    })
    .level(log::LevelFilter::Debug)
    .level_for("salsa", log::LevelFilter::Warn)
    .level_for("lsp_server", log::LevelFilter::Info)
    .chain(fern::log_file(dir.join("mc-lsp.log")).unwrap())
    .apply()
    .unwrap();

  // Copied the stdlibs panic hook, but uses `error!()` instead of stdout.
  std::panic::set_hook(Box::new(|info| {
    let location = info.location().unwrap_or_else(|| std::panic::Location::caller());

    let msg = match info.payload().downcast_ref::<&'static str>() {
      Some(s) => *s,
      None => match info.payload().downcast_ref::<String>() {
        Some(s) => &s[..],
        None => "Box<dyn Any>",
      },
    };

    let thread = std::thread::current();
    let name = thread.name().unwrap_or("<unnamed>");

    error!("thread '{name}' panicked at {location}:\n{msg}");
  }));
}
