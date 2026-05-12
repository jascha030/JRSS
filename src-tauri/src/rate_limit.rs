use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};
use std::time::{Duration, Instant};
use url::Url;

const MIN_REQUEST_INTERVAL: Duration = Duration::from_secs(2);

/// Block the current thread if the same domain was requested too recently.
/// Uses a global map so all threads coordinate (feed refresh, audio download,
/// reader extraction, etc.).
pub fn throttle_request(url: &str) {
    let domain = Url::parse(url)
        .ok()
        .and_then(|u| u.host_str().map(|h| h.to_lowercase()))
        .unwrap_or_else(|| "unknown".to_string());

    static LAST_REQUESTS: OnceLock<Mutex<HashMap<String, Instant>>> = OnceLock::new();
    let last_requests = LAST_REQUESTS.get_or_init(|| Mutex::new(HashMap::new()));

    let sleep_duration = {
        let map = last_requests.lock().expect("rate limiter lock poisoned");
        let now = Instant::now();
        map.get(&domain).and_then(|last| {
            let elapsed = now.duration_since(*last);
            if elapsed < MIN_REQUEST_INTERVAL {
                Some(MIN_REQUEST_INTERVAL - elapsed)
            } else {
                None
            }
        })
    };

    if let Some(duration) = sleep_duration {
        std::thread::sleep(duration);
    }

    {
        let mut map = last_requests.lock().expect("rate limiter lock poisoned");
        map.insert(domain, Instant::now());
    }
}
