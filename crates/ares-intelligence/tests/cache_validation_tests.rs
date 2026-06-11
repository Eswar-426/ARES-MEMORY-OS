use ares_intelligence::cache::manager::CacheManager;
use std::time::Duration;
use tokio::time::sleep;

#[tokio::test]
async fn test_cache_set_and_get() {
    let cache: CacheManager<String, String> = CacheManager::new(Duration::from_secs(60));

    cache.set("model_A_profile".to_string(), "cached_data".to_string());

    let res = cache.get(&"model_A_profile".to_string());
    assert_eq!(res, Some("cached_data".to_string()));
}

#[tokio::test]
async fn test_cache_miss() {
    let cache: CacheManager<String, String> = CacheManager::new(Duration::from_secs(60));

    let res = cache.get(&"non_existent".to_string());
    assert_eq!(res, None);
}

#[tokio::test]
async fn test_cache_ttl_expiration() {
    // TTL of 50ms
    let cache: CacheManager<String, String> = CacheManager::new(Duration::from_millis(50));

    cache.set("key".to_string(), "value".to_string());

    // Immediate get should work
    assert_eq!(cache.get(&"key".to_string()), Some("value".to_string()));

    // Wait for expiration
    sleep(Duration::from_millis(60)).await;

    // Get should return None after expiration
    assert_eq!(cache.get(&"key".to_string()), None);
}

#[tokio::test]
async fn test_cache_invalidation() {
    let cache: CacheManager<String, String> = CacheManager::new(Duration::from_secs(60));

    cache.set("key1".to_string(), "value1".to_string());
    cache.set("key2".to_string(), "value2".to_string());

    cache.invalidate(&"key1".to_string());

    assert_eq!(cache.get(&"key1".to_string()), None);
    assert_eq!(cache.get(&"key2".to_string()), Some("value2".to_string()));

    cache.invalidate_all();

    assert_eq!(cache.get(&"key2".to_string()), None);
}

#[tokio::test]
async fn test_cache_update_overwrites_ttl() {
    let cache: CacheManager<String, String> = CacheManager::new(Duration::from_millis(50));

    cache.set("key".to_string(), "value1".to_string());

    // Sleep partially
    sleep(Duration::from_millis(30)).await;

    // Overwrite with new value and fresh TTL
    cache.set("key".to_string(), "value2".to_string());

    // Sleep beyond original TTL but within new TTL
    sleep(Duration::from_millis(30)).await;

    // Should still exist because TTL was refreshed
    assert_eq!(cache.get(&"key".to_string()), Some("value2".to_string()));
}
