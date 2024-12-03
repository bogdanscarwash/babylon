import os
import re
from github import Github

# Retrieve environment variables
token = os.getenv('GH_PAT')
repo_name = os.getenv('REPO_NAME')
project_name = os.getenv('PROJECT_NAME')

# Authenticate with GitHub
g = Github(token)
repo = g.get_repo(repo_name)

# Load TODO.md content
with open('docs/TODO.md', 'r') as f:
    todo_content = f.read()

# Parse TODO.md to extract tasks
# Example parsing implementation
task_pattern = r'- \[( |x)\] (.+)'
tasks = re.findall(task_pattern, todo_content)

# Get the project by name
projects = repo.get_projects()
project = None
for p in projects:
    if p.name == project_name:
        project = p
        break

if not project:
    raise Exception(f'Project "{project_name}" not found')

# Get columns in the project
columns = {column.name: column for column in project.get_columns()}

# Map task status to columns
status_to_column = {
    'todo': columns.get('To Do'),
    'in_progress': columns.get('In Progress'),
    'done': columns.get('Done'),
}

# Label to identify issues created by the script
label_name = 'auto-todo'

# Ensure the label exists
labels = repo.get_labels()
label = None
for l in labels:
    if l.name == label_name:
        label = l
        break

if not label:
    label = repo.create_label(name=label_name, color='0E8A16')

# Get existing issues created by this script
existing_issues = repo.get_issues(labels=[label], state='all')
existing_issues_dict = {issue.title: issue for issue in existing_issues}

# Process tasks
for status_indicator, task_title in tasks:
    task_title = task_title.strip()

    # Determine task status
    if status_indicator == ' ':
        status = 'todo'
    elif status_indicator == 'x':
        status = 'done'
    else:
        status = 'todo'

    # Check if issue already exists
    issue = existing_issues_dict.get(task_title)

    if not issue:
        # Create a new issue
        issue = repo.create_issue(
            title=task_title,
            labels=[label],
        )
        # Add issue to the corresponding column
        if status_to_column[status]:
            status_to_column[status].create_card(content_id=issue.id, content_type='Issue')
    else:
        # Update issue state if necessary
        if status == 'done' and issue.state != 'closed':
            issue.edit(state='closed')
        elif status != 'done' and issue.state == 'closed':
            issue.edit(state='open')

        # Move card to the appropriate column
        # Find the project card associated with the issue
        for project_card in issue.get_project_cards():
            if project_card.project_id == project.id:
                if project_card.column_id != status_to_column[status].id:
                    project_card.move(position='top', column_id=status_to_column[status].id)
                break
        else:
            # If no card exists, create one
            if status_to_column[status]:
                status_to_column[status].create_card(content_id=issue.id, content_type='Issue')

# Optionally, close issues that are not in TODO.md anymore
titles_in_todo = set(task_title for _, task_title in tasks)
for issue in existing_issues:
    if issue.title not in titles_in_todo and issue.state != 'closed':
        issue.edit(state='closed')
