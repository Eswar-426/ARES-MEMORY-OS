import urllib.request
import urllib.error
import json

def test_strict_headers():
    print("Testing Strict Headers (Non-Strict Mode)")
    
    url = "http://127.0.0.1:8080/api/v1/knowledge/entities"
    headers = {
        "Content-Type": "application/json",
        # Missing X-ARES-STRICT header
        "Authorization": "Bearer test-token"
    }
    payload = {
        "entity_type": "Requirement",
        "name": "Test Requirement Non-Strict",
        "properties": {}
    }
    
    req = urllib.request.Request(url, data=json.dumps(payload).encode('utf-8'), headers=headers, method='POST')
    
    try:
        response = urllib.request.urlopen(req)
        status = response.status
        print(f"Status: {status}")
        assert status == 200, f"Expected HTTP 200, got {status}"
        
        warnings_header = response.getheader("X-ARES-WARNINGS")
        assert warnings_header is None, "Did not expect X-ARES-WARNINGS header in non-strict mode"
        print("Strict Headers test passed.")
    except urllib.error.HTTPError as e:
        print(f"Failed: Expected HTTP 200, got {e.code}")
        print(e.read().decode('utf-8'))
        exit(1)

if __name__ == "__main__":
    test_strict_headers()
