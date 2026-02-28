pub mod engine;
pub mod state;

use std::io::{self, Stdout};
use std::path::Path;
use std::time::Duration;

use anyhow::Result;
use crossterm::event::{
    self, DisableMouseCapture, EnableMouseCapture, Event, KeyEventKind, MouseEventKind,
};
use crossterm::execute;
use crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::Terminal;
use ratatui::backend::CrosstermBackend;

use crate::app::engine::Engine;
use crate::app::state::CompletionCard;
use crate::coach::build_coach;
use crate::content::loader::load_steps_from_root;
use crate::progress::store::ProgressStore;
use crate::terminal::guard::{GuardDecision, evaluate_command};
use crate::terminal::pty::PtySession;
use crate::terminal::shell_mode::{ShellMode, ShellRouter, run_shell_connector};
use crate::ui::popup::PopupMessage;
use crate::ui::ui_action::UiAction;
use crate::ui::ui_screen::{ScreenState, UiScreen};
use crate::verify::checks::{VerifyOutcome, run_verify};

pub fn run() -> Result<()> {
    run_with_args(std::env::args().skip(1).collect())
}

pub fn run_with_args(args: Vec<String>) -> Result<()> {
    if let Some((pin, port)) = parse_shell_connect_args(&args)? {
        return run_shell_connector(&pin, port);
    }

    let steps = load_steps_from_root(Path::new("docs/cka-app-content"))?;
    let store = ProgressStore::default()?;
    let progress = store.load()?;
    let has_progress = !progress.completed.is_empty();
    let mut engine = Engine::new(steps, progress);
    let pty = PtySession::start("/bin/zsh")?;
    let mut shell = ShellRouter::new(
        parse_shell_mode(&args)?,
        pty,
        parse_terminal_override(&args),
    );
    let coach = build_coach();

    let initial_status = format!(
        "Loaded {} steps. Shell={} Use /help. Progress file: {}",
        engine.steps.len(),
        shell.mode().as_str(),
        store.path().display()
    );

    let mut ui = UiScreen::new(initial_status, has_progress);
    if let Some(connect_cmd) = shell.external_connect_command() {
        ui.learning
            .output_log
            .push(format!("External shell pairing command: {connect_cmd}"));
        ui.learning.status =
            "External shell mode: run pairing command in another terminal".to_string();
    }

    let mut terminal = setup_terminal()?;
    let result = run_loop(
        &mut terminal,
        &mut engine,
        &mut shell,
        &store,
        coach.as_ref(),
        &mut ui,
    );
    restore_terminal(&mut terminal)?;
    result
}

fn run_loop(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    engine: &mut Engine,
    shell: &mut ShellRouter,
    store: &ProgressStore,
    coach: &(dyn crate::coach::CoachAdvisor + Send + Sync),
    ui: &mut UiScreen,
) -> Result<()> {
    let mut mouse_position: Option<(u16, u16)> = None;

    loop {
        if ui.state == ScreenState::Learning {
            ui.learning.output_log.extend(shell.drain_output());
            trim_output(&mut ui.learning.output_log, 200);
        }

        ui.update(engine)?;

        terminal.draw(|frame| {
            let _ = ui.render(frame, engine, mouse_position);
        })?;

        if !event::poll(Duration::from_millis(100))? {
            continue;
        }

        let event = event::read()?;

        let action = match event {
            Event::Mouse(mouse) => {
                mouse_position = Some((mouse.column, mouse.row));
                match mouse.kind {
                    MouseEventKind::Moved => None,
                    _ => ui.callback_registry().resolve_mouse_event(
                        mouse.kind,
                        mouse.column,
                        mouse.row,
                    ),
                }
            }
            Event::Key(key) if key.kind == KeyEventKind::Press => ui
                .callback_registry()
                .resolve_key_event(key.code)
                .or_else(|| ui.handle_key_events(key, engine)),
            _ => None,
        };

        let Some(action) = action else {
            continue;
        };

        match action {
            UiAction::None => {}
            UiAction::Quit => {
                store.save(&engine.progress)?;
                break;
            }
            UiAction::StartSession => {
                ui.transition_to_learning();
            }
            UiAction::NewSession => {
                engine.reset_progress();
                store.save(&engine.progress)?;
                ui.learning.command_input.clear();
                ui.learning.hint_message = None;
                ui.learning.completion_card = None;
                ui.learning.tab_index = 0;
                ui.learning.status = format!("Started new session at {}", engine.current_step().id);
                ui.learning
                    .output_log
                    .push("Started a new session. Progress has been reset.".to_string());
            }
            UiAction::NextStep => {
                engine.next_step();
                store.save(&engine.progress)?;
                ui.learning.status = format!("Moved to {}", engine.current_step().id);
                ui.learning.hint_message = None;
                ui.learning.completion_card = None;
                ui.learning.tab_index = 0;
            }
            UiAction::PrevStep => {
                engine.prev_step();
                store.save(&engine.progress)?;
                ui.learning.status = format!("Moved to {}", engine.current_step().id);
                ui.learning.hint_message = None;
                ui.learning.completion_card = None;
                ui.learning.tab_index = 0;
            }
            UiAction::JumpBack => {
                engine.jump_prev_completed();
                store.save(&engine.progress)?;
                ui.learning.status = format!("Jumped back to {}", engine.current_step().id);
                ui.learning.hint_message = None;
                ui.learning.completion_card = None;
                ui.learning.tab_index = 0;
            }
            UiAction::JumpRecommended => {
                engine.jump_recommended();
                store.save(&engine.progress)?;
                ui.learning.status = format!("Recommended next: {}", engine.current_step().id);
                ui.learning.hint_message = None;
                ui.learning.completion_card = None;
                ui.learning.tab_index = 0;
            }
            UiAction::Verify => match run_verify(engine.current_step())? {
                VerifyOutcome::Pass => {
                    let readiness_before = engine.readiness;
                    let finished = engine.current_step().clone();
                    let finished_id = engine.current_step().id.clone();
                    engine.complete_current();
                    store.save(&engine.progress)?;
                    let readiness_after = engine.readiness;

                    let next_commands = engine
                        .current_step()
                        .run_commands
                        .iter()
                        .take(2)
                        .cloned()
                        .collect::<Vec<_>>();
                    let what_changed = if finished.what_changed.is_empty() {
                        vec!["Deterministic checks passed".to_string()]
                    } else {
                        finished.what_changed.into_iter().take(2).collect()
                    };

                    ui.push_popup(PopupMessage::StepComplete {
                        title: format!("{} complete.", finished.title),
                        what_changed: what_changed.clone(),
                        next_commands: next_commands.clone(),
                        readiness_before,
                        readiness_after,
                    });

                    ui.learning.completion_card = Some(CompletionCard {
                        done: format!("{} complete.", finished.title),
                        what_changed,
                        next_commands,
                        verify_optional: engine
                            .current_step()
                            .verify_commands
                            .iter()
                            .take(2)
                            .cloned()
                            .collect(),
                    });
                    ui.learning.status = format!(
                        "Done: {} verified. Next: {}",
                        finished_id,
                        engine.current_step().id
                    );
                    ui.learning.hint_message = None;
                    ui.learning.tab_index = 0;
                }
                VerifyOutcome::NoChecks => {
                    ui.learning.status =
                        "No verify commands defined for this step. Use docs output.".to_string();
                    ui.learning.hint_message = None;
                    ui.learning.completion_card = None;
                    ui.learning.tab_index = 0;
                }
                VerifyOutcome::Fail(msg) => {
                    ui.push_popup(PopupMessage::VerifyFail {
                        message: msg.clone(),
                    });
                    ui.learning.status = format!("Not yet: {}", msg);
                    ui.learning.hint_message = None;
                    ui.learning.completion_card = None;
                    ui.learning.tab_index = 0;
                }
            },
            UiAction::Hint => {
                ui.learning.hint_message =
                    Some(coach.hint(engine.current_step(), &ui.learning.output_log));
                ui.learning.status = "Hint generated for current step.".to_string();
            }
            UiAction::Suggest(index) => {
                let cmds = &engine.current_step().run_commands;
                if cmds.is_empty() {
                    ui.learning.status = "No suggested run commands for this step.".to_string();
                } else if let Some(idx) = index {
                    if idx >= 1 && idx <= cmds.len() {
                        ui.learning.command_input = cmds[idx - 1].clone();
                        ui.learning.status = format!("Loaded suggestion {}/{}.", idx, cmds.len());
                        ui.learning.tab_index = idx % cmds.len();
                    } else {
                        ui.learning.status =
                            format!("Invalid suggestion index. Use 1..{}.", cmds.len());
                    }
                } else {
                    let selected = ui.learning.tab_index % cmds.len();
                    ui.learning.command_input = cmds[selected].clone();
                    ui.learning.tab_index += 1;
                    ui.learning.status =
                        format!("Loaded suggestion {}/{}.", selected + 1, cmds.len());
                }
            }
            UiAction::ClearLog => {
                ui.learning.output_log.clear();
                ui.learning
                    .output_log
                    .push("Terminal log cleared. Type /help for available commands.".to_string());
                ui.learning.status = "Cleared terminal output feed.".to_string();
            }
            UiAction::ShowHelp => {
                ui.push_popup(PopupMessage::help());
            }
            UiAction::ShowShellMode => {
                ui.learning.status = format!("Shell mode: {}", shell.mode().as_str());
            }
            UiAction::SetShellMode(mode) => match ShellMode::parse(&mode) {
                Some(parsed) => {
                    shell.set_mode(parsed);
                    ui.learning.status = format!("Shell mode switched to {}", parsed.as_str());
                    ui.learning.output_log.push(format!(
                        "Shell mode switch: now using {} runner",
                        parsed.as_str()
                    ));
                    if parsed == ShellMode::External {
                        if let Some(connect_cmd) = shell.external_connect_command() {
                            ui.learning
                                .output_log
                                .push(format!("External shell pairing command: {connect_cmd}"));
                        }
                    }
                }
                None => {
                    ui.learning.status =
                        "Unknown shell mode. Use /shell embedded or /shell external".to_string();
                }
            },
            UiAction::DismissPopup => {
                ui.dismiss_popup();
            }
            UiAction::SetCommandInput(cmd) => {
                ui.learning.command_input = cmd;
                ui.learning.status = "Command loaded. Press Enter to run.".to_string();
            }
            UiAction::RunCommand(raw_cmd) => {
                engine.record_attempt();
                match evaluate_command(&raw_cmd) {
                    GuardDecision::Allow => {
                        let dispatch = shell.send_line(&raw_cmd)?;
                        ui.learning.status =
                            format!("Ran [{}]: {}", shell.mode().as_str(), raw_cmd);
                        ui.learning.output_log.push(dispatch);
                    }
                    GuardDecision::Confirm(msg) => {
                        ui.learning.status = format!("{} (prefix with '! ' to confirm)", msg);
                    }
                    GuardDecision::Block(msg) => {
                        ui.learning.status = format!("Blocked: {}", msg);
                    }
                }
                store.save(&engine.progress)?;
                ui.learning.hint_message = None;
                ui.learning.completion_card = None;
            }
            UiAction::ForceRunCommand(raw_cmd) => {
                engine.record_attempt();
                match evaluate_command(&raw_cmd) {
                    GuardDecision::Allow | GuardDecision::Confirm(_) => {
                        let dispatch = shell.send_line(&raw_cmd)?;
                        ui.learning.status =
                            format!("Forced run [{}]: {}", shell.mode().as_str(), raw_cmd);
                        ui.learning.output_log.push(dispatch);
                    }
                    GuardDecision::Block(msg) => {
                        ui.learning.status = format!("Blocked: {}", msg);
                    }
                }
                store.save(&engine.progress)?;
                ui.learning.hint_message = None;
                ui.learning.completion_card = None;
            }
        }
    }

    Ok(())
}

fn parse_shell_connect_args(args: &[String]) -> Result<Option<(String, u16)>> {
    let mut pin: Option<String> = None;
    let mut port: Option<u16> = None;

    let mut idx = 0;
    while idx < args.len() {
        match args[idx].as_str() {
            "--shell-connect" => {
                let Some(value) = args.get(idx + 1) else {
                    anyhow::bail!("missing value for --shell-connect (expected 4-digit pin)");
                };
                pin = Some(value.clone());
                idx += 1;
            }
            "--port" => {
                let Some(value) = args.get(idx + 1) else {
                    anyhow::bail!("missing value for --port");
                };
                port = Some(
                    value
                        .parse::<u16>()
                        .map_err(|_| anyhow::anyhow!("invalid --port value '{value}'"))?,
                );
                idx += 1;
            }
            _ => {}
        }
        idx += 1;
    }

    match (pin, port) {
        (Some(pin), Some(port)) => Ok(Some((pin, port))),
        (None, None) => Ok(None),
        (Some(_), None) => anyhow::bail!("--shell-connect requires --port"),
        (None, Some(_)) => anyhow::bail!("--port provided without --shell-connect"),
    }
}

fn parse_shell_mode(args: &[String]) -> Result<ShellMode> {
    let mut idx = 0;
    while idx < args.len() {
        if args[idx] == "--shell" {
            if let Some(value) = args.get(idx + 1) {
                if let Some(mode) = ShellMode::parse(value) {
                    return Ok(mode);
                }
                anyhow::bail!(
                    "invalid --shell value '{}'; expected embedded|external",
                    value
                );
            }
            anyhow::bail!("missing value for --shell (expected embedded|external)");
        }
        idx += 1;
    }

    Ok(ShellMode::External)
}

fn parse_terminal_override(args: &[String]) -> Option<String> {
    let mut idx = 0;
    while idx < args.len() {
        if args[idx] == "--terminal" {
            return args.get(idx + 1).cloned();
        }
        idx += 1;
    }

    parse_positional_terminal(args)
}

fn parse_positional_terminal(args: &[String]) -> Option<String> {
    let mut skip_next = false;

    for arg in args {
        if skip_next {
            skip_next = false;
            continue;
        }

        match arg.as_str() {
            "--shell" | "--terminal" | "--shell-connect" | "--port" => {
                skip_next = true;
            }
            value if value.starts_with('-') => {}
            value => return Some(value.to_string()),
        }
    }

    None
}

fn trim_output(output_log: &mut Vec<String>, max: usize) {
    if output_log.len() > max {
        let drop = output_log.len().saturating_sub(max);
        output_log.drain(0..drop);
    }
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.hide_cursor()?;
    Ok(terminal)
}

fn restore_terminal(terminal: &mut Terminal<CrosstermBackend<Stdout>>) -> Result<()> {
    let _ = disable_raw_mode();
    let _ = execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture);
    let _ = terminal.show_cursor();
    Ok(())
}
