import urllib.request
import urllib.error
import json

def test_missing_owner():
    print("Testing Missing Owner (Strict Mode)")
    
    url = "http://127.0.0.1:8080/api/v1/knowledge/entities"
    headers = {
        "Content-Type": "application/json",
        "X-ARES-STRICT": "true",
        "Authorization": "Bearer test-token"
    }
    # This payload is missing the has_owner logic
    payload = {
        "entity_type": "Requirement",
        "name": "Test Requirement Missing Owner",
        "properties": {}
    }
    
    req = urllib.request.Request(url, data=json.dumps(payload).encode('utf-8'), headers=headers, method='POST')
    
    try:
        response = urllib.request.urlopen(req)
        print(f"Failed: Expected HTTP 422, got {response.status}")
        print(response.read().decode('utf-8'))
        exit(1)
    except urllib.error.HTTPError as e:
        if e.code == 422:
            raw_body = e.read().decode('utf-8')
            data = json.loads(raw_body)
            violations = data.get("violations", [])
            has_ownership_violation = any(v.get("policy_name") == "OWNERSHIP-001" for v in violations)
            assert has_ownership_violation, f"Expected OWNERSHIP-001 violation, got {violations}"
            print("Missing Owner test passed.")
        else:
            print(f"Failed: Expected HTTP 422, got {e.code}")
            print(e.read().decode('utf-8'))
            exit(1)

if __name__ == "__main__":
    test_missing_owner()
