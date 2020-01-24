//! HTTP Method filters.
//!
//! The filters deal with the HTTP Method part of a request. Several here will
//! match the request `Method`, and if not matched, will reject the request
//! with a `405 Method Not Allowed`.
//!
//! There is also [`portal::method()`](method), which never rejects
//! a request, and just extracts the method to be used in your filter chains.
use futures::future;
use http::Method;

use crate::filter::{filter_fn, filter_fn_one, Filter, One};
use crate::reject::Rejection;
use std::convert::Infallible;

/// Create a `Filter` that requires the request method to be `GET`.
///
/// # Example
///
/// ```
/// use portal::Filter;
///
/// let get_only = portal::get().map(portal::reply);
/// ```
pub fn get() -> impl Filter<Extract = (), Error = Rejection> + Copy {
    method_is(|| &Method::GET)
}

/// Create a `Filter` that requires the request method to be `POST`.
///
/// # Example
///
/// ```
/// use portal::Filter;
///
/// let post_only = portal::post().map(portal::reply);
/// ```
pub fn post() -> impl Filter<Extract = (), Error = Rejection> + Copy {
    method_is(|| &Method::POST)
}

/// Create a `Filter` that requires the request method to be `PUT`.
///
/// # Example
///
/// ```
/// use portal::Filter;
///
/// let put_only = portal::put().map(portal::reply);
/// ```
pub fn put() -> impl Filter<Extract = (), Error = Rejection> + Copy {
    method_is(|| &Method::PUT)
}

/// Create a `Filter` that requires the request method to be `DELETE`.
///
/// # Example
///
/// ```
/// use portal::Filter;
///
/// let delete_only = portal::delete().map(portal::reply);
/// ```
pub fn delete() -> impl Filter<Extract = (), Error = Rejection> + Copy {
    method_is(|| &Method::DELETE)
}

/// Create a `Filter` that requires the request method to be `HEAD`.
///
/// # Example
///
/// ```
/// use portal::Filter;
///
/// let head_only = portal::head().map(portal::reply);
/// ```
pub fn head() -> impl Filter<Extract = (), Error = Rejection> + Copy {
    method_is(|| &Method::HEAD)
}

/// Create a `Filter` that requires the request method to be `OPTIONS`.
///
/// # Example
///
/// ```
/// use portal::Filter;
///
/// let options_only = portal::options().map(portal::reply);
/// ```
pub fn options() -> impl Filter<Extract = (), Error = Rejection> + Copy {
    method_is(|| &Method::OPTIONS)
}

/// Create a `Filter` that requires the request method to be `PATCH`.
///
/// # Example
///
/// ```
/// use portal::Filter;
///
/// let patch_only = portal::patch().map(portal::reply);
/// ```
pub fn patch() -> impl Filter<Extract = (), Error = Rejection> + Copy {
    method_is(|| &Method::PATCH)
}

/// Extract the `Method` from the request.
///
/// This never rejects a request.
///
/// # Example
///
/// ```
/// use portal::Filter;
///
/// let route = portal::method()
///     .map(|method| {
///         format!("You sent a {} request!", method)
///     });
/// ```
pub fn method() -> impl Filter<Extract = One<Method>, Error = Infallible> + Copy {
    filter_fn_one(|route| future::ok::<_, Infallible>(route.method().clone()))
}

// NOTE: This takes a static function instead of `&'static Method` directly
// so that the `impl Filter` can be zero-sized. Moving it around should be
// cheaper than holding a single static pointer (which would make it 1 word).
fn method_is<F>(func: F) -> impl Filter<Extract = (), Error = Rejection> + Copy
where
    F: Fn() -> &'static Method + Copy,
{
    filter_fn(move |route| {
        let method = func();
        log::trace!("method::{:?}?: {:?}", method, route.method());
        if route.method() == method {
            future::ok(())
        } else {
            future::err(crate::reject::method_not_allowed())
        }
    })
}

#[cfg(test)]
mod tests {
    #[test]
    fn method_size_of() {
        // See comment on `method_is` function.
        assert_eq!(std::mem::size_of_val(&super::get()), 0,);
    }
}
