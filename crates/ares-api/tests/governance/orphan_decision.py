import urllib.request
import urllib.error
import json
import sys

def test_orphan_decision():
    print("Testing Orphan Decision (Strict Mode)")
    
    url = "http://127.0.0.1:8080/api/v1/knowledge/entities"
    headers = {
        "Content-Type": "application/json",
        "X-ARES-STRICT": "true",
        "Authorization": "Bearer test-token"
    }
    payload = {
        "entity_type": "Decision",
        "name": "Test Decision",
        "properties": {}
    }
    
    req = urllib.request.Request(url, data=json.dumps(payload).encode('utf-8'), headers=headers, method='POST')
    
    try:
        response = urllib.request.urlopen(req)
        print(f"Failed: Expected HTTP 422, got {response.status}")
        print(response.read().decode('utf-8'))
        exit(1)
    except urllib.error.HTTPError as e:
        print(f"Status Code: {e.code}")
        if e.code == 422 or e.code == 400:
            raw_body = e.read().decode('utf-8')
            print(f"Raw body: {raw_body}")
            data = json.loads(raw_body)
            print(f"Response: {data}")
            assert data.get("status") == "blocked" or data.get("error") is not None, f"Expected blocked status or error, got {data}"
            print("Orphan Decision test passed.")
        else:
            print(f"Failed: Expected HTTP 422, got {e.code}")
            print(e.read().decode('utf-8'))
            exit(1)

if __name__ == "__main__":
    test_orphan_decision()
