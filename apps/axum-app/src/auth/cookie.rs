pub const SESSION_COOKIE: &str = "sid";

use time::Duration;
use tower_cookies::{Cookie, PrivateCookies};

pub fn set_session_cookie(
    cookies: &PrivateCookies<'_>,
    session_id: &str,
    max_age_secs: i64,
    secure: bool,
) {
    let mut cookie = Cookie::new(SESSION_COOKIE, session_id.to_string());
    cookie.set_http_only(true);
    cookie.set_path("/");
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
    cookie.set_max_age(Duration::seconds(max_age_secs));
    if secure {
        cookie.set_secure(true);
    }
    cookies.add(cookie);
}

pub fn clear_session_cookie(cookies: &PrivateCookies<'_>, secure: bool) {
    let mut cookie = Cookie::new(SESSION_COOKIE, "");
    cookie.set_http_only(true);
    cookie.set_path("/");
    cookie.set_same_site(tower_cookies::cookie::SameSite::Lax);
    cookie.set_max_age(Duration::seconds(0));
    if secure {
        cookie.set_secure(true);
    }
    cookies.add(cookie);
}
