import os
import re
import sys
from datetime import datetime

import requests
from github import Github


def parse_markdown(markdown_content):
    """Parse the markdown content to extract issues, labels, and milestones."""

    # Initialize storage structures
    issues = []
    labels = []
    milestones = []
    current_section = None
    current_milestone = None

    # Parse the content line by line
    lines = markdown_content.split("\n")
    for line in lines:
        # Parse Milestones
        milestone_match = re.match(r"### 🎯 (.+)", line)
        if milestone_match:
            title = milestone_match.group(1).split(":")[0].strip()
            # Extract due date if it exists in the following lines
            try:
                due_date_line = next(
                    l for l in lines[lines.index(line) :] if "Due:" in l
                )
                due_date = due_date_line.split("Due:")[1].strip()
                due_date = datetime.strptime(f"{due_date}-01", "%Q%Y-%d")
            except:
                due_date = None

            current_milestone = {
                "title": title,
                "due_on": due_date.isoformat() if due_date else None,
                "description": "",
            }
            milestones.append(current_milestone)
            continue

        # Parse Labels
        if line.startswith("- `") and "`" in line:
            label_text = line.split("`")[1]
            description = line.split("- `")[1].split("`")[1].strip()
            if description.startswith("- "):
                description = description[2:]
            labels.append(
                {
                    "name": label_text,
                    "description": description,
                    "color": "0366d6",  # Default GitHub blue
                }
            )
            continue

        # Parse Issues
        issue_match = re.match(r"- \[ \] #(\d+) (.+)", line)
        if issue_match:
            issue_number, title = issue_match.groups()

            # Extract labels from the section header
            current_labels = []
            if current_section:
                label_matches = re.findall(r"\[(.*?)\]", current_section)
                current_labels.extend(label_matches)

            issues.append(
                {
                    "title": title,
                    "number": int(issue_number),
                    "labels": current_labels,
                    "milestone": (
                        current_milestone["title"] if current_milestone else None
                    ),
                    "body": "",  # You can add more context from surrounding text if needed
                }
            )
            continue

        # Track current section for context
        if line.startswith("###"):
            current_section = line[4:].strip()

    return {"issues": issues, "labels": labels, "milestones": milestones}


def create_github_project(
    token, repo_owner, repo_name, project_title="Babylon Development Board"
):
    """Create a GitHub Project and return its ID."""

    headers = {
        "Authorization": f"token {token}",
        "Accept": "application/vnd.github.v3+json",
    }

    # Create the project using the REST API
    url = f"https://api.github.com/repos/{repo_owner}/{repo_name}/projects"
    data = {
        "name": project_title,
        "body": "Automated project board created from Kanban markdown",
    }

    response = requests.post(url, headers=headers, json=data)
    if response.status_code != 201:
        raise Exception(f"Failed to create project: {response.text}")

    return response.json()["id"]


def sync_to_github(markdown_content, token, repo_owner, repo_name):
    """Sync the Kanban board to GitHub Projects."""

    # Parse the markdown content
    parsed_data = parse_markdown(markdown_content)

    # Initialize GitHub client
    g = Github(token)
    repo = g.get_repo(f"{repo_owner}/{repo_name}")

    # Create labels
    existing_labels = {label.name: label for label in repo.get_labels()}
    for label_data in parsed_data["labels"]:
        if label_data["name"] not in existing_labels:
            repo.create_label(**label_data)
            print(f"Created label: {label_data['name']}")

    # Create milestones
    existing_milestones = {m.title: m for m in repo.get_milestones()}
    for milestone_data in parsed_data["milestones"]:
        if milestone_data["title"] not in existing_milestones:
            repo.create_milestone(**milestone_data)
            print(f"Created milestone: {milestone_data['title']}")

    # Create project board
    project_id = create_github_project(token, repo_owner, repo_name)

    # Create columns (To Do, In Progress, Done)
    columns = {}
    for column_name in ["To Do", "In Progress", "Done"]:
        url = f"https://api.github.com/projects/{project_id}/columns"
        headers = {
            "Authorization": f"token {token}",
            "Accept": "application/vnd.github.v3+json",
        }
        response = requests.post(url, headers=headers, json={"name": column_name})
        columns[column_name] = response.json()["id"]

    # Create issues and add them to the project
    for issue_data in parsed_data["issues"]:
        # Create the issue
        labels = [repo.get_label(label) for label in issue_data["labels"]]
        milestone = (
            repo.get_milestone(issue_data["milestone"])
            if issue_data["milestone"]
            else None
        )

        issue = repo.create_issue(
            title=issue_data["title"],
            body=issue_data["body"],
            labels=labels,
            milestone=milestone,
        )
        print(f"Created issue: {issue.title}")

        # Add issue to project (in To Do column by default)
        url = f'https://api.github.com/projects/columns/{columns["To Do"]}/cards'
        headers = {
            "Authorization": f"token {token}",
            "Accept": "application/vnd.github.v3+json",
        }
        data = {"content_id": issue.id, "content_type": "Issue"}
        requests.post(url, headers=headers, json=data)


if __name__ == "__main__":
    # Get GitHub token from environment variable
    token = os.getenv("GITHUB_TOKEN")
    if not token:
        print("Please set the GITHUB_TOKEN environment variable")
        sys.exit(1)

    # Read markdown file
    with open("kanban.md") as f:
        markdown_content = f.read()

    # Get repository details
    repo_owner = input("Enter repository owner: ")
    repo_name = input("Enter repository name: ")

    # Sync to GitHub
    sync_to_github(markdown_content, token, repo_owner, repo_name)
    print("Kanban board successfully synced to GitHub Projects!")
