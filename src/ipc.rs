use hyprland::data::*;
use hyprland::dispatch::{
    Dispatch, DispatchType as DT, MonitorIdentifier, WorkspaceIdentifier,
    WorkspaceIdentifierWithSpecial,
};
use hyprland::prelude::*;
use hyprland::shared::HyprError;
use hyprland::Result;

fn trace(verbose: bool, msg: impl std::fmt::Display) {
    if verbose {
        eprintln!("hyprqtile: {msg}");
    }
}

pub struct MonitorsResult {
    pub active_monitor: i128,
    pub passive_monitor: Option<i128>,
    pub monitors: Vec<Monitor>,
}

impl MonitorsResult {
    pub fn get(workspace_id: i32) -> Result<Self> {
        let monitors = Monitors::get()?.to_vec();
        let mut active_monitor = 1;
        let mut passive_monitor = None;

        for monitor in &monitors {
            if monitor.focused {
                active_monitor = monitor.id;
                continue;
            }
            if monitor.active_workspace.id == workspace_id {
                passive_monitor = Some(monitor.id);
            }
        }

        Ok(Self {
            active_monitor,
            passive_monitor,
            monitors: monitors.to_vec(),
        })
    }
}

pub fn move_to(workspace_id: i32, verbose: bool) -> Result<()> {
    trace(
        verbose,
        format!("target workspace {workspace_id} (querying monitors)"),
    );
    let monitors = MonitorsResult::get(workspace_id)?;

    if monitors.monitors.len() == 1 {
        trace(
            verbose,
            format!(
                "one monitor (id {}); switch workspace {workspace_id} then move to this monitor",
                monitors.active_monitor
            ),
        );
        switch_to_workspace(workspace_id, Some(monitors.active_monitor), verbose)?;
        return Ok(());
    }

    match monitors.passive_monitor {
        Some(passive_monitor_id) => {
            trace(
                verbose,
                format!(
                    "workspace {workspace_id} active on monitor {passive_monitor_id}; pull to focused monitor {}",
                    monitors.active_monitor
                ),
            );
            pull_workspace_to_focused_monitor(workspace_id, monitors.active_monitor, verbose)?
        }
        None => {
            trace(
                verbose,
                format!(
                    "workspace {workspace_id} not active on another monitor; switch/create then move to monitor {}",
                    monitors.active_monitor
                ),
            );
            switch_to_workspace(workspace_id, Some(monitors.active_monitor), verbose)?
        }
    }

    Ok(())
}

pub fn get_current_workspace() -> Result<i32> {
    Ok(Workspace::get_active()?.id)
}

pub fn move_to_next(verbose: bool) -> Result<()> {
    let from = get_current_workspace()?;
    let to = from + 1;
    trace(verbose, format!("next: {from} → {to}"));
    move_to(to, verbose)
}

pub fn move_to_previous(verbose: bool) -> Result<()> {
    let from = get_current_workspace()?;
    let to = from - 1;
    trace(verbose, format!("previous: {from} → {to}"));
    move_to(to, verbose)
}

fn dispatch_workspace_by_id(workspace_id: i32, verbose: bool) -> Result<()> {
    let wsp_special = WorkspaceIdentifierWithSpecial::Id(workspace_id);
    trace(verbose, format!("dispatch workspace {workspace_id}"));
    match Dispatch::call(DT::Workspace(wsp_special)) {
        Ok(()) => {}
        Err(HyprError::NotOkDispatch(val))
            if val == "Previous workspace doesn't exist".to_owned() =>
        {
            // harmless Hyprland quirk; workspace still switches or is created
        }
        Err(e) => return Err(e),
    }
    Ok(())
}

fn pull_workspace_to_focused_monitor(
    workspace_id: i32,
    active_monitor_id: i128,
    verbose: bool,
) -> Result<()> {
    trace(
        verbose,
        format!("dispatch moveworkspacetomonitor {workspace_id} → monitor {active_monitor_id}"),
    );
    let wsp = WorkspaceIdentifier::Id(workspace_id);
    let mon = MonitorIdentifier::Id(active_monitor_id);
    Dispatch::call(DT::MoveWorkspaceToMonitor(wsp, mon))?;
    dispatch_workspace_by_id(workspace_id, verbose)?;
    Ok(())
}

pub fn switch_to_workspace(
    workspace_id: i32,
    active_monitor_id: Option<i128>,
    verbose: bool,
) -> Result<()> {
    dispatch_workspace_by_id(workspace_id, verbose)?;

    if let Some(active_monitor_id) = active_monitor_id {
        trace(
            verbose,
            format!("dispatch moveworkspacetomonitor {workspace_id} → monitor {active_monitor_id}"),
        );
        let wsp = WorkspaceIdentifier::Id(workspace_id);
        let mon = MonitorIdentifier::Id(active_monitor_id);
        Dispatch::call(DT::MoveWorkspaceToMonitor(wsp, mon))?;
    }

    Ok(())
}
