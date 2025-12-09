#!/usr/bin/env python3
"""
GitHub Issue Migration Script

Migrates all issues from feagi-data-processing to feagi-core repository.

Usage:
    python3 scripts/migrate_issues.py

Requirements:
    - GitHub Personal Access Token (PAT) with 'repo' scope
    - Set GITHUB_TOKEN environment variable or pass via --token flag
    - requests library: pip install requests

The script will:
    1. Fetch all issues from feagi/feagi-data-processing
    2. Create corresponding issues in feagi/feagi-core
    3. Preserve labels, state (open/closed), and comments
    4. Add migration note to each issue body
"""

import os
import sys
import time
import argparse
import json
from typing import Dict, List, Optional, Any
from datetime import datetime

try:
    import requests
except ImportError:
    print("Error: 'requests' library not found.")
    print("Install it with: pip install requests")
    sys.exit(1)

# GitHub API base URL
GITHUB_API_BASE = "https://api.github.com"

# Source and destination repositories
SOURCE_REPO = "feagi/feagi-data-processing"
DEST_REPO = "feagi/feagi-core"

# Rate limiting: GitHub allows 5000 requests/hour for authenticated requests
# We'll be conservative and add small delays
REQUEST_DELAY = 0.5  # seconds between requests


class IssueMigrator:
    """Handles migration of GitHub issues between repositories."""

    def __init__(self, token: str, dry_run: bool = False):
        """
        Initialize the migrator.

        Args:
            token: GitHub Personal Access Token
            dry_run: If True, only print what would be done without making changes
        """
        self.token = token
        self.dry_run = dry_run
        self.session = requests.Session()
        self.session.headers.update({
            "Authorization": f"token {token}",
            "Accept": "application/vnd.github.v3+json",
            "User-Agent": "FEAGI-Issue-Migrator"
        })
        self.migrated_count = 0
        self.failed_count = 0
        self.skipped_count = 0
        self.migration_log: List[Dict[str, Any]] = []

    def _make_request(self, method: str, url: str, **kwargs) -> Optional[requests.Response]:
        """
        Make an API request with error handling and rate limiting.

        Args:
            method: HTTP method (GET, POST, etc.)
            url: Full URL or path relative to API base
            **kwargs: Additional arguments for requests

        Returns:
            Response object or None if error
        """
        if not url.startswith("http"):
            url = f"{GITHUB_API_BASE}/{url.lstrip('/')}"

        try:
            time.sleep(REQUEST_DELAY)
            response = self.session.request(method, url, **kwargs)
            response.raise_for_status()
            return response
        except requests.exceptions.HTTPError as e:
            if response.status_code == 404:
                print(f"  Warning: Resource not found: {url}")
            elif response.status_code == 403:
                print(f"  Error: Forbidden (check token permissions): {url}")
                if "rate limit" in response.text.lower():
                    print("  Rate limit exceeded. Waiting 60 seconds...")
                    time.sleep(60)
                    return self._make_request(method, url, **kwargs)
            else:
                print(f"  Error: HTTP {response.status_code}: {e}")
            return None
        except requests.exceptions.RequestException as e:
            print(f"  Error: Request failed: {e}")
            return None

    def get_all_issues(self, repo: str, state: str = "all") -> List[Dict[str, Any]]:
        """
        Fetch all issues from a repository.

        Args:
            repo: Repository in format 'owner/repo'
            state: Issue state ('open', 'closed', or 'all')

        Returns:
            List of issue dictionaries
        """
        print(f"Fetching {state} issues from {repo}...")
        issues = []
        page = 1
        per_page = 100

        while True:
            url = f"repos/{repo}/issues"
            params = {
                "state": state,
                "page": page,
                "per_page": per_page,
                "sort": "created",
                "direction": "asc"
            }

            response = self._make_request("GET", url, params=params)
            if not response:
                break

            page_issues = response.json()
            if not page_issues:
                break

            # Filter out pull requests (they have 'pull_request' key)
            page_issues = [issue for issue in page_issues if "pull_request" not in issue]
            issues.extend(page_issues)

            print(f"  Fetched page {page}: {len(page_issues)} issues (total: {len(issues)})")

            # Check if there are more pages
            if len(page_issues) < per_page:
                break

            page += 1

        print(f"Total issues found: {len(issues)}")
        return issues

    def get_issue_comments(self, repo: str, issue_number: int) -> List[Dict[str, Any]]:
        """
        Fetch all comments for an issue.

        Args:
            repo: Repository in format 'owner/repo'
            issue_number: Issue number

        Returns:
            List of comment dictionaries
        """
        comments = []
        page = 1
        per_page = 100

        while True:
            url = f"repos/{repo}/issues/{issue_number}/comments"
            params = {"page": page, "per_page": per_page}

            response = self._make_request("GET", url, params=params)
            if not response:
                break

            page_comments = response.json()
            if not page_comments:
                break

            comments.extend(page_comments)

            if len(page_comments) < per_page:
                break

            page += 1

        return comments

    def create_issue(
        self,
        repo: str,
        title: str,
        body: str,
        labels: List[str],
        state: str = "open"
    ) -> Optional[Dict[str, Any]]:
        """
        Create a new issue in the destination repository.

        Args:
            repo: Repository in format 'owner/repo'
            title: Issue title
            body: Issue body
            labels: List of label names
            state: Issue state ('open' or 'closed')

        Returns:
            Created issue dictionary or None if error
        """
        url = f"repos/{repo}/issues"
        data = {
            "title": title,
            "body": body,
            "labels": labels
        }

        if self.dry_run:
            print(f"  [DRY RUN] Would create issue: {title}")
            return {"number": 0, "html_url": f"https://github.com/{repo}/issues/0"}

        response = self._make_request("POST", url, json=data)
        if not response:
            return None

        created_issue = response.json()

        # If the original issue was closed, close the new one too
        if state == "closed":
            self.close_issue(repo, created_issue["number"])

        return created_issue

    def close_issue(self, repo: str, issue_number: int) -> bool:
        """
        Close an issue.

        Args:
            repo: Repository in format 'owner/repo'
            issue_number: Issue number

        Returns:
            True if successful, False otherwise
        """
        url = f"repos/{repo}/issues/{issue_number}"
        data = {"state": "closed"}

        if self.dry_run:
            print(f"  [DRY RUN] Would close issue #{issue_number}")
            return True

        response = self._make_request("PATCH", url, json=data)
        return response is not None

    def create_comment(self, repo: str, issue_number: int, body: str) -> bool:
        """
        Create a comment on an issue.

        Args:
            repo: Repository in format 'owner/repo'
            issue_number: Issue number
            body: Comment body

        Returns:
            True if successful, False otherwise
        """
        url = f"repos/{repo}/issues/{issue_number}/comments"
        data = {"body": body}

        if self.dry_run:
            print(f"  [DRY RUN] Would add comment to issue #{issue_number}")
            return True

        response = self._make_request("POST", url, json=data)
        return response is not None

    def ensure_labels_exist(self, repo: str, labels: List[str]) -> None:
        """
        Ensure all labels exist in the destination repository.
        Creates missing labels with default color.

        Args:
            repo: Repository in format 'owner/repo'
            labels: List of label names
        """
        if not labels:
            return

        print(f"Ensuring {len(labels)} labels exist in {repo}...")

        # Get existing labels
        url = f"repos/{repo}/labels"
        response = self._make_request("GET", url, params={"per_page": 100})
        if not response:
            print("  Warning: Could not fetch existing labels")
            return

        existing_labels = {label["name"] for label in response.json()}
        missing_labels = [label for label in labels if label not in existing_labels]

        if not missing_labels:
            print(f"  All labels already exist")
            return

        # Create missing labels with default color
        default_color = "ededed"  # Light gray
        for label in missing_labels:
            url = f"repos/{repo}/labels"
            data = {
                "name": label,
                "color": default_color,
                "description": f"Migrated from {SOURCE_REPO}"
            }

            if self.dry_run:
                print(f"  [DRY RUN] Would create label: {label}")
            else:
                response = self._make_request("POST", url, json=data)
                if response:
                    print(f"  Created label: {label}")
                else:
                    print(f"  Failed to create label: {label}")

    def migrate_issue(self, issue: Dict[str, Any]) -> bool:
        """
        Migrate a single issue to the destination repository.

        Args:
            issue: Issue dictionary from source repository

        Returns:
            True if successful, False otherwise
        """
        issue_number = issue["number"]
        title = issue["title"]
        original_body = issue.get("body", "") or ""
        state = issue["state"]
        labels = [label["name"] for label in issue.get("labels", [])]
        created_at = issue.get("created_at", "")
        original_url = issue.get("html_url", "")

        print(f"\nMigrating issue #{issue_number}: {title}")

        # Add migration note to body
        migration_note = f"\n\n---\n**Migrated from [{SOURCE_REPO}#{issue_number}]({original_url})**\n"
        if created_at:
            migration_note += f"*Original issue created: {created_at}*\n"
        body = original_body + migration_note

        # Ensure labels exist
        if labels:
            self.ensure_labels_exist(DEST_REPO, labels)

        # Create the issue
        created_issue = self.create_issue(
            repo=DEST_REPO,
            title=title,
            body=body,
            labels=labels,
            state=state
        )

        if not created_issue:
            print(f"  Failed to create issue")
            self.failed_count += 1
            self.migration_log.append({
                "source_issue": issue_number,
                "title": title,
                "status": "failed",
                "error": "Failed to create issue"
            })
            return False

        new_issue_number = created_issue["number"]
        new_issue_url = created_issue.get("html_url", "")

        print(f"  Created issue #{new_issue_number}: {new_issue_url}")

        # Migrate comments
        comments = self.get_issue_comments(SOURCE_REPO, issue_number)
        if comments:
            print(f"  Migrating {len(comments)} comments...")
            for comment in comments:
                comment_body = comment.get("body", "")
                comment_author = comment.get("user", {}).get("login", "unknown")
                comment_created = comment.get("created_at", "")

                # Add attribution to comment
                comment_with_attr = (
                    f"{comment_body}\n\n"
                    f"---\n"
                    f"*Comment by @{comment_author} from original issue*\n"
                    f"*Posted: {comment_created}*\n"
                )

                if not self.create_comment(DEST_REPO, new_issue_number, comment_with_attr):
                    print(f"    Warning: Failed to migrate comment by {comment_author}")

        self.migrated_count += 1
        self.migration_log.append({
            "source_issue": issue_number,
            "source_url": original_url,
            "dest_issue": new_issue_number,
            "dest_url": new_issue_url,
            "title": title,
            "status": "success"
        })

        return True

    def migrate_all_issues(self) -> None:
        """Migrate all issues from source to destination repository."""
        print(f"Issue Migration: {SOURCE_REPO} -> {DEST_REPO}")
        print("=" * 60)

        if self.dry_run:
            print("DRY RUN MODE - No changes will be made\n")

        # Fetch all issues
        issues = self.get_all_issues(SOURCE_REPO, state="all")

        if not issues:
            print("No issues found to migrate.")
            return

        print(f"\nStarting migration of {len(issues)} issues...\n")

        # Migrate each issue
        for issue in issues:
            self.migrate_issue(issue)

        # Print summary
        print("\n" + "=" * 60)
        print("Migration Summary:")
        print(f"  Total issues: {len(issues)}")
        print(f"  Successfully migrated: {self.migrated_count}")
        print(f"  Failed: {self.failed_count}")
        print(f"  Skipped: {self.skipped_count}")

        # Save migration log
        log_file = f"migration_log_{datetime.now().strftime('%Y%m%d_%H%M%S')}.json"
        with open(log_file, "w") as f:
            json.dump(self.migration_log, f, indent=2)
        print(f"\nMigration log saved to: {log_file}")


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description="Migrate GitHub issues from feagi-data-processing to feagi-core"
    )
    parser.add_argument(
        "--token",
        help="GitHub Personal Access Token (or set GITHUB_TOKEN env var)",
        default=os.environ.get("GITHUB_TOKEN")
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Perform a dry run without making any changes"
    )

    args = parser.parse_args()

    if not args.token:
        print("Error: GitHub token required")
        print("Set GITHUB_TOKEN environment variable or use --token flag")
        print("\nTo create a token:")
        print("1. Go to https://github.com/settings/tokens")
        print("2. Generate new token (classic)")
        print("3. Select 'repo' scope")
        sys.exit(1)

    migrator = IssueMigrator(token=args.token, dry_run=args.dry_run)
    migrator.migrate_all_issues()


if __name__ == "__main__":
    main()

