use common::*;
use super::*;

use std::collections::HashMap;

use auth;
use cursor::{CursorIter, UserCursor, ListCursor};
use error::Error::TwitterError;
use links;
use user::{UserID, TwitterUser};
use tweet;

///Look up the lists the given user has been added to.
pub fn memberships<'a>(user: &'a UserID, token: &'a auth::Token) -> CursorIter<'a, ListCursor> {
    let mut params = HashMap::new();
    add_name_param(&mut params, user);
    CursorIter::new(links::lists::MEMBERSHIPS, token, Some(params), Some(20))
}

///Return up to 100 lists the given user is subscribed to, including those the user made
///themselves.
///
///TODO: this is not strictly `subscriptions` and `ownerships` blended
pub fn list<'a>(user: &'a UserID, owned_first: bool, token: &'a auth::Token)
    -> WebResponse<Vec<List>>
{
    let mut params = HashMap::new();
    add_name_param(&mut params, user);
    add_param(&mut params, "reverse", owned_first.to_string());

    let mut resp = try!(auth::get(links::lists::LIST, token, Some(&params)));
    parse_response(&mut resp)
}

///Look up the lists the given user is subscribed to, but not ones the user made themselves.
pub fn subscriptions<'a>(user: &'a UserID, token: &'a auth::Token) -> CursorIter<'a, ListCursor> {
    let mut params = HashMap::new();
    add_name_param(&mut params, user);
    CursorIter::new(links::lists::SUBSCRIPTIONS, token, Some(params), Some(20))
}

///Look up the lists created by the given user.
pub fn ownerships<'a>(user: &'a UserID, token: &'a auth::Token) -> CursorIter<'a, ListCursor> {
    let mut params = HashMap::new();
    add_name_param(&mut params, user);
    CursorIter::new(links::lists::OWNERSHIPS, token, Some(params), Some(20))
}

///Look up information for a single list.
pub fn show<'a>(list: ListID<'a>, token: &'a auth::Token) -> WebResponse<List> {
    let mut params = HashMap::new();

    add_list_param(&mut params, &list);

    let mut resp = try!(auth::get(links::lists::SHOW, token, Some(&params)));

    parse_response(&mut resp)
}

///Look up the users that have been added to the given list.
pub fn members<'a>(list: ListID<'a>, token: &'a auth::Token) -> CursorIter<'a, UserCursor> {
    let mut params = HashMap::new();

    add_list_param(&mut params, &list);

    CursorIter::new(links::lists::MEMBERS, token, Some(params), Some(20))
}

///Look up the users that have subscribed to the given list.
pub fn subscribers<'a>(list: ListID<'a>, token: &'a auth::Token) -> CursorIter<'a, UserCursor> {
    let mut params = HashMap::new();

    add_list_param(&mut params, &list);

    CursorIter::new(links::lists::SUBSCRIBERS, token, Some(params), Some(20))
}

///Check whether the given user is subscribed to the given list.
pub fn is_subscribed<'a, T: Into<UserID<'a>>>(user: T, list: ListID<'a>, token: &auth::Token) ->
    WebResponse<bool>
{
    let mut params = HashMap::new();

    add_list_param(&mut params, &list);
    add_name_param(&mut params, &user.into());

    let mut resp = try!(auth::get(links::lists::IS_SUBSCRIBER, token, Some(&params)));

    let out: WebResponse<TwitterUser> = parse_response(&mut resp);

    match out {
        Ok(user) => Ok(Response::map(user, |_| true)),
        Err(TwitterError(terrs)) => {
            if terrs.errors.iter().any(|e| e.code == 109) {
                //here's a fun conundrum: since "is not in this list" is returned as an error code,
                //the rate limit info that would otherwise be part of the response isn't there. the
                //rate_headers method was factored out specifically for this location, since it's
                //still there, just accompanying an error response instead of a user.
                Ok(Response::map(rate_headers(&resp), |_| false))
            }
            else {
                Err(TwitterError(terrs))
            }
        },
        Err(err) => Err(err),
    }
}

///Check whether the given user has been added to the given list.
pub fn is_member<'a, T: Into<UserID<'a>>>(user: T, list: ListID<'a>, token: &auth::Token) ->
    WebResponse<bool>
{
    let mut params = HashMap::new();

    add_list_param(&mut params, &list);
    add_name_param(&mut params, &user.into());

    let mut resp = try!(auth::get(links::lists::IS_MEMBER, token, Some(&params)));

    let out: WebResponse<TwitterUser> = parse_response(&mut resp);

    match out {
        Ok(user) => Ok(Response::map(user, |_| true)),
        Err(TwitterError(terrs)) => {
            if terrs.errors.iter().any(|e| e.code == 109) {
                //here's a fun conundrum: since "is not in this list" is returned as an error code,
                //the rate limit info that would otherwise be part of the response isn't there. the
                //rate_headers method was factored out specifically for this location, since it's
                //still there, just accompanying an error response instead of a user.
                Ok(Response::map(rate_headers(&resp), |_| false))
            }
            else {
                Err(TwitterError(terrs))
            }
        },
        Err(err) => Err(err),
    }
}

///Begin navigating the collection of tweets made by the users added to the given list.
pub fn statuses<'a>(list: ListID<'a>, with_rts: bool, token: &'a auth::Token)
    -> tweet::Timeline<'a>
{
    let mut params = HashMap::new();
    add_list_param(&mut params, &list);
    add_param(&mut params, "include_rts", with_rts.to_string());

    tweet::Timeline::new(links::lists::STATUSES, Some(params), token)
}

///Adds the given user to the given list.
pub fn add<'a, T: Into<UserID<'a>>>(list: ListID<'a>, user: T, token: &auth::Token)
    -> WebResponse<List>
{
    let mut params = HashMap::new();
    add_list_param(&mut params, &list);
    add_name_param(&mut params, &user.into());

    let mut resp = try!(auth::post(links::lists::ADD, token, Some(&params)));

    parse_response(&mut resp)
}
