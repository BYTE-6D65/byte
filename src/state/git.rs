use std::path::PathBuf;

/// Git repository status
#[derive(Debug, Clone)]
pub struct GitStatus {
    pub is_repo: bool,
    pub branch: Option<String>,
    pub is_clean: bool,
    pub modified: usize,
    pub staged: usize,
    pub untracked: usize,
    pub ahead: usize,
    pub behind: usize,
}

impl GitStatus {
    /// Create a status indicating the directory is not a git repository
    pub fn not_a_repo() -> Self {
        Self {
            is_repo: false,
            branch: None,
            is_clean: true,
            modified: 0,
            staged: 0,
            untracked: 0,
            ahead: 0,
            behind: 0,
        }
    }

    /// Create a status indicating an error occurred
    pub fn error() -> Self {
        Self {
            is_repo: true,
            branch: None,
            is_clean: true,
            modified: 0,
            staged: 0,
            untracked: 0,
            ahead: 0,
            behind: 0,
        }
    }
}

impl Default for GitStatus {
    fn default() -> Self {
        Self {
            is_repo: true,
            branch: None,
            is_clean: true,
            modified: 0,
            staged: 0,
            untracked: 0,
            ahead: 0,
            behind: 0,
        }
    }
}

/// Get git status for a project directory
pub fn get_git_status(project_path: &str) -> GitStatus {
    // Fast exit: check if .git directory exists
    let git_dir = PathBuf::from(project_path).join(".git");
    if !git_dir.exists() {
        return GitStatus::not_a_repo();
    }

    // Run git status command
    match run_git_status_command(project_path) {
        Ok(output) => parse_git_status(&output),
        Err(_) => GitStatus::error(),
    }
}

/// Run git status command and capture output using exec API
fn run_git_status_command(project_path: &str) -> Result<String, std::io::Error> {
    use crate::exec::CommandBuilder;

    let result = CommandBuilder::git("status")
        .arg("--porcelain=v1")
        .arg("--branch")
        .working_dir(project_path)
        .execute()
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))?;

    if result.success {
        Ok(result.stdout)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "git command failed",
        ))
    }
}

/// Parse git status porcelain output
fn parse_git_status(output: &str) -> GitStatus {
    let mut status = GitStatus::default();

    for line in output.lines() {
        if line.starts_with("##") {
            // Parse branch line
            parse_branch_line(line, &mut status);
        } else if line.starts_with("??") {
            // Untracked file
            status.untracked += 1;
        } else if line.len() >= 2 {
            // Check staged status (first character)
            let first_char = line.chars().nth(0);
            if first_char.is_some() && first_char != Some(' ') && first_char != Some('?') {
                status.staged += 1;
            }

            // Check unstaged status (second character)
            let second_char = line.chars().nth(1);
            if second_char.is_some() && second_char != Some(' ') && second_char != Some('?') {
                status.modified += 1;
            }
        }
    }

    // Determine if repository is clean
    status.is_clean = status.modified == 0 && status.staged == 0 && status.untracked == 0;

    status
}

/// Parse the branch line from git status output
/// Format: ## branch...remote [ahead X, behind Y]
fn parse_branch_line(line: &str, status: &mut GitStatus) {
    // Remove "## " prefix
    let line = line.trim_start_matches("## ");

    // Handle initial commit case
    if line.starts_with("No commits yet on ") {
        let branch = line.trim_start_matches("No commits yet on ");
        status.branch = Some(branch.to_string());
        return;
    }

    // Handle detached HEAD
    if line.contains("HEAD (no branch)") || line.starts_with("HEAD detached") {
        status.branch = None; // Will show as "(detached HEAD)"
        return;
    }

    // Parse branch name and tracking info
    if let Some(dots_pos) = line.find("...") {
        // Branch with remote tracking
        let branch = &line[..dots_pos];
        status.branch = Some(branch.to_string());

        // Parse ahead/behind from the rest of the line
        let rest = &line[dots_pos + 3..];

        // Look for [ahead X, behind Y] or [ahead X] or [behind Y]
        if let Some(bracket_start) = rest.find('[') {
            if let Some(bracket_end) = rest.find(']') {
                let tracking_info = &rest[bracket_start + 1..bracket_end];

                // Parse ahead
                if let Some(ahead_pos) = tracking_info.find("ahead ") {
                    let ahead_str = &tracking_info[ahead_pos + 6..];
                    if let Some(comma_or_end) =
                        ahead_str.find(',').or_else(|| Some(ahead_str.len()))
                    {
                        if let Ok(ahead) = ahead_str[..comma_or_end].trim().parse::<usize>() {
                            status.ahead = ahead;
                        }
                    }
                }

                // Parse behind
                if let Some(behind_pos) = tracking_info.find("behind ") {
                    let behind_str = &tracking_info[behind_pos + 7..];
                    if let Some(comma_or_end) =
                        behind_str.find(',').or_else(|| Some(behind_str.len()))
                    {
                        if let Ok(behind) = behind_str[..comma_or_end].trim().parse::<usize>() {
                            status.behind = behind;
                        }
                    }
                }
            }
        }
    } else {
        // Branch without remote tracking (local branch)
        // Remove any trailing indicators
        let branch = line.split_whitespace().next().unwrap_or(line);
        status.branch = Some(branch.to_string());
    }
}
