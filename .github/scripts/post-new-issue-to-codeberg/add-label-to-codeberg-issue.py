from datetime import datetime, timezone
import json
import os
import sys
import urllib.request

DRY_RUN = os.environ.get("DRY_RUN", "false") == "true"

data = {
    "labels": ["GitHub"],
    "updated_at": datetime.now(timezone.utc).isoformat(),
}
data_json = json.dumps(data)

req = urllib.request.Request(
    method="POST",
    url=f"https://codeberg.org/api/v1/repos/{os.environ['CODEBERG_REPO']}/issues/{os.environ['CODEBERG_ISSUE_NUMBER']}/labels",
    headers={
        "Authorization": f"token {os.environ['CODEBERG_TOKEN']}",
        "Content-Type": "application/json",
    },
    data=data_json.encode("utf-8"),
)

if DRY_RUN:
    print("DRY RUN: Skipping API call", file=sys.stderr)
    print("POST DATA:", data_json)
else:
    try:
        with urllib.request.urlopen(req) as resp:
            body = resp.read().decode()
            print("Response Code:", resp.getcode(), file=sys.stderr)
            print(body)
    except urllib.error.HTTPError as e:
        print("Failed add GitHub label to issue:", e.read().decode(), file=sys.stderr)
        raise
