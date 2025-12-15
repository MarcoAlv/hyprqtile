use hyprland::data::*;
use hyprland::dispatch::{
    Dispatch, DispatchType as DT, MonitorIdentifier, WorkspaceIdentifier,
    WorkspaceIdentifierWithSpecial,
};
use hyprland::prelude::*;
use hyprland::shared::HyprError;
use hyprland::Result;

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

pub fn move_to(workspace_id: i32) -> Result<()> {
    let monitors = MonitorsResult::get(workspace_id)?;

    if monitors.monitors.len() == 1 {
        switch_to_workspace(workspace_id, None)?;
        return Ok(());
    }

    match monitors.passive_monitor {
        Some(passive_monitor_id) => {
            swap_active_workspace(monitors.active_monitor, passive_monitor_id)?
        }
        None => {
            // target is not shown anywhere → switch and auto-create if missing
            switch_to_workspace(workspace_id, Some(monitors.active_monitor))?
        }
    }

    Ok(())
}

pub fn get_current_workspace() -> Result<i32> {
    Ok(Workspace::get_active()?.id)
}

pub fn move_to_next() -> Result<()> {
    move_to(get_current_workspace()? + 1)
}

pub fn move_to_previous() -> Result<()> {
    move_to(get_current_workspace()? - 1)
}

pub fn swap_active_workspace(active_monitor_id: i128, passive_monitor_id: i128) -> Result<()> {
    let active = MonitorIdentifier::Id(active_monitor_id);
    let passive = MonitorIdentifier::Id(passive_monitor_id);
    Dispatch::call(DT::SwapActiveWorkspaces(active, passive))?;
    Ok(())
}

pub fn switch_to_workspace(workspace_id: i32, active_monitor_id: Option<i128>) -> Result<()> {
    let wsp_special = WorkspaceIdentifierWithSpecial::Id(workspace_id);

    // If a monitor is given, try moving the workspace to that monitor.
    if let Some(active_monitor_id) = active_monitor_id {
        let wsp = WorkspaceIdentifier::Id(workspace_id);
        let mon = MonitorIdentifier::Id(active_monitor_id);

        match Dispatch::call(DT::MoveWorkspaceToMonitor(wsp, mon)) {
            // Workspace does not exist → ignore and continue.
            Err(HyprError::NotOkDispatch(val))
                if val == "moveWorkspaceToMonitor workspace doesn't exist!" =>
            {
                // fall-through: DT::Workspace will create it
            }
            Err(e) => return Err(e),
            _ => {}
        }
    }

    // This dispatch always succeeds in creating the workspace if missing.
    match Dispatch::call(DT::Workspace(wsp_special)) {
        Ok(()) => Ok(()),
        Err(HyprError::NotOkDispatch(val))
            if val == "Previous workspace doesn't exist".to_owned() =>
        {
            // harmless, workspace still gets created
            Ok(())
        }
        Err(e) => Err(e),
    }
}
