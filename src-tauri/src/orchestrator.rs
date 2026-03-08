use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

use chrono::Utc;
use rusqlite::{params, Connection};
use tauri::State;

use crate::{
    models::{
        CommandResponse, DeployRequest, DeployResult, EnvInjectionRequest, ManagedProject,
    },
    state::{detect_bun_status, AppState},
};

#[tauri::command]
pub fn list_managed_projects(state: State<'_, AppState>) -> Result<Vec<ManagedProject>, String> {
    let connection = Connection::open(&state.paths.database_path).map_err(|error| error.to_string())?;
    let mut statement = connection
        .prepare(
            "
            SELECT id, slug, source_url, workspace_path, branch, status, created_at, updated_at
            FROM managed_projects
            ORDER BY updated_at DESC
            ",
        )
        .map_err(|error| error.to_string())?;

    let rows = statement
        .query_map([], |row| {
            Ok(ManagedProject {
                id: row.get(0)?,
                slug: row.get(1)?,
                source_url: row.get(2)?,
                workspace_path: row.get(3)?,
                branch: row.get(4)?,
                status: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .map_err(|error| error.to_string())?;

    let mut projects = Vec::new();
    for row in rows {
        projects.push(row.map_err(|error| error.to_string())?);
    }

    Ok(projects)
}

#[tauri::command]
pub fn deploy_to_pc(
    request: DeployRequest,
    state: State<'_, AppState>,
) -> Result<DeployResult, String> {
    if !detect_bun_status().installed {
        return Err(
            "Bun is not installed yet. Run the Bun installer from RalphHub before deploying."
                .to_string(),
        );
    }

    let normalized_url = normalize_repo_url(&request.url)?;
    let slug = slug_from_url(&normalized_url);
    let workspace = state.paths.repos_dir.join(&slug);

    if workspace.exists() && workspace.join(".git").exists() {
        run_command(
            "git",
            &["pull", "--ff-only"],
            Some(&workspace),
            "Failed to update existing workspace",
        )?;
    } else {
        let workspace_arg = workspace.display().to_string();
        run_command(
            "git",
            &["clone", normalized_url.as_str(), workspace_arg.as_str()],
            None,
            "Failed to clone repository",
        )?;
    }

    if !workspace.join("package.json").exists() {
        return Err(
            "This repository does not contain a package.json. RalphHub will not fall back to npm."
                .to_string(),
        );
    }

    run_command(
        "bun",
        &["install"],
        Some(&workspace),
        "bun install failed for the managed workspace",
    )?;

    let branch = current_branch(&workspace).unwrap_or_else(|| "main".to_string());
    let state_path = ensure_state_file(&workspace)?;
    let env_path = workspace.join(".env");

    register_project(
        &state,
        &slug,
        &normalized_url,
        &workspace,
        &branch,
        "ready",
    )?;

    Ok(DeployResult {
        workspace_path: workspace.display().to_string(),
        normalized_url,
        branch,
        message: "Repository cloned and initialized with Bun.".to_string(),
        state_path: state_path.display().to_string(),
        env_path: env_path.display().to_string(),
        notebook_path: None,
    })
}

#[tauri::command]
pub fn deploy_to_colab(
    request: DeployRequest,
    state: State<'_, AppState>,
) -> Result<DeployResult, String> {
    let normalized_url = normalize_repo_url(&request.url)?;
    let slug = slug_from_url(&normalized_url);
    let notebook_path = state
        .paths
        .notebooks_dir
        .join(format!("{slug}-overnight.ipynb"));
    let state_path = state
        .paths
        .state_dir
        .join(format!("{slug}-colab-state.md"));
    let env_path = state.paths.state_dir.join(format!("{slug}.env"));

    if !state_path.exists() {
        fs::write(
            &state_path,
            format!(
                "# RalphHub Colab State\n\n- Repo: {normalized_url}\n- Generated: {}\n",
                Utc::now().to_rfc3339()
            ),
        )
        .map_err(|error| error.to_string())?;
    }

    let notebook = serde_json::json!({
        "nbformat": 4,
        "nbformat_minor": 5,
        "metadata": {
            "kernelspec": {
                "display_name": "Python 3",
                "language": "python",
                "name": "python3"
            },
            "language_info": {
                "name": "python"
            }
        },
        "cells": [
            {
                "cell_type": "code",
                "metadata": {},
                "source": [format!("!git clone {normalized_url}\n")],
                "outputs": [],
                "execution_count": null
            },
            {
                "cell_type": "code",
                "metadata": {},
                "source": [
                    format!("%cd {slug}\n"),
                    "!curl -fsSL https://bun.sh/install | bash\n",
                    "!export BUN_INSTALL=\"$HOME/.bun\"\n",
                    "!export PATH=\"$BUN_INSTALL/bin:$PATH\"\n",
                    "!bun install\n"
                ],
                "outputs": [],
                "execution_count": null
            },
            {
                "cell_type": "code",
                "metadata": {},
                "source": [
                    "# Fill these values with the same keys saved in RalphHub Settings before running.\n",
                    "ANTHROPIC_API_KEY = ''\n",
                    "OPENAI_API_KEY = ''\n",
                    "GROK_API_KEY = ''\n",
                    "GEMINI_API_KEY = ''\n",
                    "PERPLEXICA_KEYS = ''\n"
                ],
                "outputs": [],
                "execution_count": null
            },
            {
                "cell_type": "code",
                "metadata": {},
                "source": [
                    "# Start the overnight Ralph loop for the selected model/workflow.\n",
                    "!bun run start-ralph-loop || true\n"
                ],
                "outputs": [],
                "execution_count": null
            }
        ]
    });

    fs::write(
        &notebook_path,
        serde_json::to_string_pretty(&notebook).map_err(|error| error.to_string())?,
    )
    .map_err(|error| error.to_string())?;

    Ok(DeployResult {
        workspace_path: String::new(),
        normalized_url,
        branch: "main".to_string(),
        message: "Colab notebook generated successfully.".to_string(),
        state_path: state_path.display().to_string(),
        env_path: env_path.display().to_string(),
        notebook_path: Some(notebook_path.display().to_string()),
    })
}

#[tauri::command]
pub fn inject_keys(request: EnvInjectionRequest) -> Result<CommandResponse, String> {
    let workspace = PathBuf::from(&request.workspace_path);
    if !workspace.exists() {
        return Err(format!("Workspace does not exist: {}", request.workspace_path));
    }

    let env_example_path = workspace.join(".env.example");
    let env_path = workspace.join(".env");
    let mut contents = if env_example_path.exists() {
        fs::read_to_string(&env_example_path).map_err(|error| error.to_string())?
    } else {
        String::new()
    };

    for entry in request.entries {
        let line = format!("{}={}", entry.key, shell_escape_env_value(&entry.value));
        if contents.contains(&format!("{}=", entry.key)) {
            contents = replace_env_line(&contents, &entry.key, &line);
        } else {
            if !contents.ends_with('\n') && !contents.is_empty() {
                contents.push('\n');
            }
            contents.push_str(&line);
            contents.push('\n');
        }
    }

    fs::write(&env_path, contents).map_err(|error| error.to_string())?;

    Ok(CommandResponse {
        ok: true,
        message: format!("Injected keys into {}.", env_path.display()),
    })
}

fn register_project(
    state: &AppState,
    slug: &str,
    source_url: &str,
    workspace_path: &Path,
    branch: &str,
    status: &str,
) -> Result<(), String> {
    let connection = Connection::open(&state.paths.database_path).map_err(|error| error.to_string())?;
    let timestamp = Utc::now().to_rfc3339();
    let id = format!("{slug}:{branch}");

    connection
        .execute(
            "
            INSERT INTO managed_projects (id, slug, source_url, workspace_path, branch, status, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
            ON CONFLICT(id) DO UPDATE SET
                source_url = excluded.source_url,
                workspace_path = excluded.workspace_path,
                status = excluded.status,
                updated_at = excluded.updated_at
            ",
            params![
                id,
                slug,
                source_url,
                workspace_path.display().to_string(),
                branch,
                status,
                timestamp,
                timestamp
            ],
        )
        .map_err(|error| error.to_string())?;

    Ok(())
}

fn ensure_state_file(workspace: &Path) -> Result<PathBuf, String> {
    let path = workspace.join("STATE.md");
    if !path.exists() {
        fs::write(
            &path,
            "# RalphHub State\n\n- Status: initialized\n- Next step: update this file from the active workflow.\n",
        )
        .map_err(|error| error.to_string())?;
    }

    Ok(path)
}

fn run_command(
    program: &str,
    args: &[&str],
    current_dir: Option<&Path>,
    failure_message: &str,
) -> Result<(), String> {
    let mut command = Command::new(program);
    command.args(args);

    if let Some(current_dir) = current_dir {
        command.current_dir(current_dir);
    }

    let output = command.output().map_err(|error| error.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!(
            "{failure_message}: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ))
    }
}

fn normalize_repo_url(url: &str) -> Result<String, String> {
    let trimmed = url.trim();
    if trimmed.is_empty() {
        return Err("Repository URL is required.".to_string());
    }

    if trimmed.contains("github.com/") || trimmed.contains("huggingface.co/") {
        Ok(trimmed.trim_end_matches('/').to_string())
    } else {
        Err("Only GitHub and Hugging Face repository URLs are supported.".to_string())
    }
}

fn slug_from_url(url: &str) -> String {
    url.split('/')
        .next_back()
        .unwrap_or("workspace")
        .trim_end_matches(".git")
        .replace('.', "-")
}

fn current_branch(workspace: &Path) -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(workspace)
        .output()
        .ok()?;

    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_string())
}

fn replace_env_line(contents: &str, key: &str, replacement: &str) -> String {
    contents
        .lines()
        .map(|line| {
            if line.trim_start().starts_with(&format!("{key}=")) {
                replacement.to_string()
            } else {
                line.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn shell_escape_env_value(value: &str) -> String {
    if value.contains(' ') || value.contains('"') {
        format!("\"{}\"", value.replace('"', "\\\""))
    } else {
        value.to_string()
    }
}
