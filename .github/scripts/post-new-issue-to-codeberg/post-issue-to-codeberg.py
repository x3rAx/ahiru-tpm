import json
import os
import sys
import urllib.request

DRY_RUN = os.environ.get("DRY_RUN", "false") == "true"

body = """\
> [!IMPORTANT]
>
> **This issue has been mirrored from GitHub:**
> Posted by: @${github_issue_author}
> 👉 [View on GitHub]({github_issue_url})

{github_issue_body}
""".format(
    **{
        "github_issue_url": os.environ["GITHUB_ISSUE_URL"],
        "github_issue_author": os.environ["GITHUB_ISSUE_AUTHOR"],
        "github_issue_body": os.environ["GITHUB_ISSUE_BODY"],
    }
)

data = {
    "title": os.environ["GITHUB_ISSUE_TITLE"],
    "body": body,
}
data_json = json.dumps(data)

req = urllib.request.Request(
    method="POST",
    url=f"https://codeberg.org/api/v1/repos/{os.environ['CODEBERG_REPO']}/issues",
    headers={
        "Content-Type": "application/json",
        "Authorization": f"token {os.environ['CODEBERG_TOKEN']}",
    },
    data=data_json.encode("utf-8"),
)

if DRY_RUN:
    print("DRY RUN: Skipping API call", file=sys.stderr)
    print("POST DATA:", data_json)
    # Print dummy output
    with open(os.environ["GITHUB_OUTPUT"], "a") as output:
        print("codeberg_issue_number=1", file=output)
        print("codeberg_issue_url=https://example.tld/owner/repo/issues/1", file=output)
else:
    try:
        with urllib.request.urlopen(req) as resp:
            print("Issue created on Codeberg:", resp.status, file=sys.stderr)
            response = json.load(resp)
            with open(os.environ["GITHUB_OUTPUT"], "a") as output:
                print(f"codeberg_issue_number={response['number']}", file=output)
                print(f"codeberg_issue_url={response['html_url']}", file=output)
            issue_number = response["number"]
    except urllib.error.HTTPError as e:
        print("Failed to create issue:", e.read().decode(), file=sys.stderr)
        raise
