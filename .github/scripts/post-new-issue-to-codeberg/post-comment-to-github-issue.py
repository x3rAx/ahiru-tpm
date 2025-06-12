import json
import os
import sys
import urllib.request

DRY_RUN = os.environ.get("DRY_RUN", "false") == "true"

body = """\
> [!Note]
>
> **This issue has been mirrored to Codeberg:**
> 👉 [View on Codeberg]({codeberg_issue_url})

*This is an automated answer.*
""".format(
    **{
        "codeberg_issue_url": os.environ["CODEBERG_ISSUE_URL"],
    }
)

data = {"body": body}
data_json = json.dumps(data)

req = urllib.request.Request(
    method="POST",
    url=f"https://api.github.com/repos/{os.environ['GITHUB_REPO']}/issues/{os.environ['GITHUB_ISSUE_NUMBER']}/comments",
    headers={
        "Authorization": f"token {os.environ['GITHUB_TOKEN']}",
        "Accept": "application/vnd.github+json",
        "Content-Type": "application/json",
        "User-Agent": "GitHub-Issue-Mirror",
    },
    data=data_json.encode("utf-8"),
)

if DRY_RUN:
    print("DRY RUN: Skipping API call", file=sys.stderr)
    print("POST DATA:", data_json)
    # Print dummy output
    with open(os.environ["GITHUB_OUTPUT"], "a") as output:
        print("codeberg_issue_url=https://example.tld/owner/repo/issues/1", file=output)
else:
    try:
        with urllib.request.urlopen(req) as resp:
            body = resp.read().decode()
            print("Response Status:", resp.status, file=sys.stderr)
            print(body)
    except urllib.error.HTTPError as e:
        print("Failed to comment:", e.read().decode(), file=sys.stderr)
        raise
