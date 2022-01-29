use std::{error::Error, path::Path, process::Command};

/// Runs `git pull` in the given directory
pub fn execute_pull_request(path: &Path) -> Result<PullResult, Box<dyn Error>> {
    let output = Command::new("git").arg("pull").current_dir(path).output()?;
    if output.status.success() {
        Ok(check_pull_result(std::str::from_utf8(&output.stdout)?))
    } else {
        Ok(check_pull_result(std::str::from_utf8(&output.stderr)?))
    }
}

fn check_pull_result(git_output: &str) -> PullResult {
    if git_output.contains("Already up to date.") {
        PullResult::UpToDate
    } else if git_output.contains("Updating") {
        PullResult::Success
    } else {
        PullResult::Failed(git_output.to_string())
    }
}

#[derive(Debug, PartialEq)]
pub enum PullResult {
    Failed(String),
    Success,
    UpToDate,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_pull_up_to_date() {
        let output = "Already up to date.

";
        let value: PullResult = check_pull_result(output);
        assert_eq!(PullResult::UpToDate, value);
    }

    #[test]
    fn test_check_pull_success() {
        let output = "remote: Enumerating objects: 5, done.
remote: Counting objects: 100% (5/5), done.
Unpacking objects: 100% (3/3), 630 bytes | 45.00 KiB/s, done.
remote: Total 3 (delta 0), reused 0 (delta 0), pack-reused 0
From github.com:paunstefan/test_repo
    5a837f4..4c38a65  main       -> origin/main
Updating 5a837f4..4c38a65
Fast-forward
    README.md | 4 +++-
    1 file changed, 3 insertions(+), 1 deletion(-)

    ";
        let value: PullResult = check_pull_result(output);
        assert_eq!(PullResult::Success, value);
    }

    #[test]
    fn test_check_pull_failed_failed() {
        let output = "remote: Enumerating objects: 5, done.
remote: Counting objects: 100% (5/5), done.
Unpacking objects: 100% (3/3), 634 bytes | 57.00 KiB/s, done.
remote: Total 3 (delta 0), reused 0 (delta 0), pack-reused 0
From github.com:paunstefan/test_repo
    4c38a65..e86551b  main       -> origin/main
hint: You have divergent branches and need to specify how to reconcile them.
hint: You can do so by running one of the following commands sometime before
hint: your next pull:
hint:
hint:   git config pull.rebase false  # merge (the default strategy)
hint:   git config pull.rebase true   # rebase
hint:   git config pull.ff only       # fast-forward only
hint:
hint: You can replace \"git config\" with \"git config --global\" to set a default
hint: preference for all repositories. You can also pass --rebase, --no-rebase,
hint: or --ff-only on the command line to override the configured default per
hint: invocation.
fatal: Need to specify how to reconcile divergent branches.

";
        let value: PullResult = check_pull_result(output);
        assert_eq!(PullResult::Failed(output.to_string()), value);
    }
}
