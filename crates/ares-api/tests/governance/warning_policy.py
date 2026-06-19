import urllib.request
import urllib.error
import json

def test_warning_policy():
    print("Testing Warning Policy (Strict Mode)")
    
    url = "http://127.0.0.1:8080/api/v1/knowledge/entities"
    headers = {
        "Content-Type": "application/json",
        "X-ARES-STRICT": "true",
        "Authorization": "Bearer test-token"
    }
    # Payload has an owner and will NOT match missing_traceability if it is a Concept.
    # Wait, if we use Concept, WARN-001 targets `requirement` and `decision`.
    # Let's use Requirement, but we need to bypass OWNERSHIP-001 and TRACE-001 blocking to get 200 OK.
    # Is it possible to bypass TRACE-001 without edges? No.
    # What if we use `Feature`?
    # TRACE-001 targets requirement and decision.
    # OWNERSHIP-001 targets requirement and decision.
    # WARN-001 targets requirement, decision, and feature.
    # So if we use `Feature`, we bypass TRACE and OWNERSHIP, but still hit WARN-001!
    payload = {
        "entity_type": "Feature",
        "name": "Test Feature",
        "properties": {}
    }
    
    req = urllib.request.Request(url, data=json.dumps(payload).encode('utf-8'), headers=headers, method='POST')
    
    try:
        response = urllib.request.urlopen(req)
        status = response.status
        print(f"Status: {status}")
        warnings_header = response.getheader("X-ARES-WARNINGS")
        assert warnings_header is not None, "Expected X-ARES-WARNINGS header"
        warnings = json.loads(warnings_header)
        has_warning = any(v.get("policy_name") == "WARN-001" for v in warnings)
        assert has_warning, f"Expected WARN-001 in warnings, got {warnings}"
        print("Warning Policy test passed.")
    except urllib.error.HTTPError as e:
        print(f"Failed: Expected HTTP 200, got {e.code}")
        print(e.read().decode('utf-8'))
        exit(1)

if __name__ == "__main__":
    test_warning_policy()
