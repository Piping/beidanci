#![feature(proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate lazy_static;

mod types;
mod db;
#[cfg(test)]
mod tests;

use rocket::{
    fairing::AdHoc,
    get,
    http::{Cookie, Cookies, RawStr},
    request::Form,
    response::{content::Plain, Flash, Redirect},
    routes, State,
};

use std::{
    collections::HashMap,
    fs::File,
    sync::atomic::{AtomicUsize, Ordering},
};

use maud::{html, Markup};
use strum::IntoEnumIterator;
use types::{PanelRankType, ServerAcceptLangauge};
use db::*;

struct HitCount(AtomicUsize);

lazy_static! {
    static ref TEXT: HashMap<ServerAcceptLangauge, HashMap<&'static str, &'static str>> = [
        (
            ServerAcceptLangauge::SimpliedChinese,
            [
                ("lang-id", "zh"),
                ("lang-name", "中文"),
                ("site-title", "背单词，记概念，值得拥有"),
                ("signup-login-button", "注册/登陆"),
            ]
            .iter()
            .copied()
            .collect()
        ),
        (
            ServerAcceptLangauge::English,
            [
                ("lang-id", "en"),
                ("lang-name", "English"),
                (
                    "site-title",
                    "Memorize foreign vocab? new concept? USE the app"
                ),
                ("signup-login-button", "Sign Up/Login"),
            ]
            .iter()
            .copied()
            .collect()
        ),
    ]
    .iter()
    .cloned()
    .collect();
}

use anyhow::{anyhow, Context, Result}; //.context() //anyhow!()

struct AppModel {
    lang: ServerAcceptLangauge,
    user_id: String,
    user_action_type: String,
    user_progress_idx: u32,
    user_vocab_book_idx: u32,
    the_word: String,
    the_word_type: String,
    the_word_meaning: String,
    flash_msg: Option<String>,
}

impl AppModel {
    fn new(lang: ServerAcceptLangauge, mut cookies: Cookies, conn: &db::Connection) -> Result<AppModel> {
        let lang = cookies
            .get("state_choosen_lang")
            .map_or(lang, |c| c.value().into());

        let user_action_type = cookies
            .get("user_action_type")
            .map_or("to_answer", |c| c.value())
            .to_string();

        let flash_msg = cookies.get("_flash").map(|c| c.value().to_string());

        let user_id = "xxx".to_string();

        let user_progress_idx = cookies.get("vocab_idx").map_or(0, |c| c.value().parse::<i64>().unwrap()) + 1;
        let user_vocab_book_idx = 0;


        let (the_word,_) = Vocab::get_by_idx(user_progress_idx as i64, &conn).ok().unwrap();
        let the_word = the_word.get(0).unwrap().vocab.clone();
        let the_word_type = "动词".to_string();
        let the_word_meaning = "欢呼".to_string();
        let user_progress_idx = user_progress_idx as u32;

        Ok(AppModel {
            lang,
            flash_msg,
            user_id,
            user_action_type,
            user_progress_idx,
            user_vocab_book_idx,
            the_word,
            the_word_type,
            the_word_meaning,
        })
    }
}

fn makrdown_parse_clean(input: &str) -> String {
    use pulldown_cmark::{html, Options, Parser};
    let mut options = Options::empty();
    options.insert(Options::ENABLE_STRIKETHROUGH);
    let parser = Parser::new_ext(input, options);
    let mut html_output = String::new();
    html::push_html(&mut html_output, parser);
    let safe_html = ammonia::clean(&*html_output);
    safe_html
}

#[get("/favicon.ico")]
fn favicon() -> Option<Plain<File>> {
    let filename = format!("static/icons/favicon.ico");
    File::open(&filename).map(|f| Plain(f)).ok()
}

#[get("/instantclick.min.js")]
fn instantclick() -> Option<Plain<File>> {
    let filename = format!("static/instantclick.min.js");
    File::open(&filename).map(|f| Plain(f)).ok()
}

#[get("/robots.txt")]
fn robots() -> &'static str {
    "
    User-agent: *
    Allow: /
    "
}

#[get("/api/set-lang?<lang>")]
fn setlang(mut cookies: Cookies, lang: &RawStr) -> Result<Redirect> {
    let cookie = Cookie::build("state_choosen_lang", lang.url_decode()?)
        .path("/")
        .secure(false)
        .finish();
    cookies.add(cookie);
    Ok(Redirect::to(uri!(index)))
}

#[get("/api/order?<rankby>")]
fn set_panel_rank(mut cookies: Cookies, rankby: &RawStr) -> Result<Redirect, std::str::Utf8Error> {
    let cookie = Cookie::build("state_choosen_rank", rankby.url_decode()?)
        .path("/")
        .secure(false)
        .finish();
    cookies.add(cookie);
    Ok(Redirect::to(uri!(index)))
}

#[get("/")]
fn index(
    lang: ServerAcceptLangauge,
    cookies: Cookies,
    hit_count: State<HitCount>,
    conn: db::Connection,
) -> Result<Markup> {
    hit_count.0.fetch_add(1, Ordering::Release);
    let model = AppModel::new(lang, cookies, &conn)?;
    Ok(default_view(&model))
}

#[derive(Debug, FromForm)]
struct UserInput {
    user_id: String,
    user_action_type: String,
    user_vocab: String,
    user_progress_idx: u32,
    user_vocab_book_idx: u32,
}

#[get("/prounciation?<vocab>")]
fn get_prounciation(
    lang: ServerAcceptLangauge,
    vocab: String,
    conn: db::Connection,
) -> Option<Plain<Vec<u8>>> {
    VocabSpeech::get_by_word(&vocab, &conn).ok().map(|f| Plain(f.mp3))
}

#[post("/iknow", data = "<user>")]
fn check_answer_when_know(
    lang: ServerAcceptLangauge,
    mut cookies: Cookies,
    user: Form<UserInput>,
) -> Result<Redirect> {
    let cookie = Cookie::build("user_action_type", "to_check")
        .path("/")
        .secure(false)
        .finish();
    cookies.add(cookie);
    Ok(Redirect::to(format!("/")))
}

#[post("/idontknow", data = "<user>")]
fn check_answer_when_dontknow(
    lang: ServerAcceptLangauge,
    mut cookies: Cookies,
    user: Form<UserInput>,
) -> Result<Redirect> {
    let cookie = Cookie::build("user_action_type", "to_remember")
        .path("/")
        .secure(false)
        .finish();
    cookies.add(cookie);
    Ok(Redirect::to(format!("/")))
}

#[post("/iamright", data = "<user>")]
fn get_next_question_when_right(
    lang: ServerAcceptLangauge,
    mut cookies: Cookies,
    user: Form<UserInput>,
) -> Result<Redirect> {
    let cookie = Cookie::build("user_action_type", "to_answer")
        .path("/")
        .secure(false)
        .finish();
    cookies.add(cookie);
    Ok(Redirect::to(format!("/")))
}

#[post("/iamwrong", data = "<user>")]
fn get_next_question_when_wrong(
    lang: ServerAcceptLangauge,
    mut cookies: Cookies,
    user: Form<UserInput>,
) -> Result<Redirect> {
    let ck_next = Cookie::build("user_action_type", "to_answer")
        .path("/")
        .secure(false)
        .finish();

    let ck_count = cookies.get("vocab_idx").map_or(0, |c| c.value().parse::<i64>().unwrap()) + 1;
    let ck_count = Cookie::build("vocab_idx", ck_count.to_string())
        .path("/")
        .secure(false)
        .finish();
    cookies.add(ck_next);
    cookies.add(ck_count);
    Ok(Redirect::to(format!("/")))
}

#[get("/hitcount")]
fn hitcount(hit_count: State<HitCount>) -> String {
    hit_count.0.load(Ordering::Relaxed).to_string()
}

fn header_view(options: &AppModel) -> Markup {
    let lang = options.lang;
    html! {
       nav class="navbar" role="navigation" aria-label="main navigation" {
       } //nav
    } //html!
}

fn toggle_active_js(id: &str) -> String {
    toggle_js(id, "is-active")
}

fn toggle_js(id: &str, class: &str) -> String {
    format!("document.getElementById('{}').classList.toggle('{}');", id, class)
}

fn keypress_js() -> Markup {
    html! {
        (maud::PreEscaped(
        r#"
          <script type="text/javascript">
            $(document).keypress(function(event) {
                console.log(event.originalEvent.key);
                if (event.originalEvent.key == 'z' || event.originalEvent.key == 'Z') {
                    $('#Z').click();
                }
                if (event.originalEvent.key == 'x' || event.originalEvent.key == 'X') {
                    $('#X').click();
                }
                if (event.originalEvent.key == 'c' || event.originalEvent.key == 'c') {
                    $('#C').click();
                }
            });
          </script>
        "#
        ))
    }
}

fn timeago_js() -> Markup {
    html! {
        (maud::PreEscaped(
        r#"
          <script type="text/javascript">
            console.log('Send your Resume!');
            // TimeAgo Configuration
            $('time.timeago').timeago();
          </script>
        "#
        ))
    }
}

fn remember_scroll_js() -> Markup {
    html! {
        (maud::PreEscaped(
        r#"
          <script type="text/javascript">
            console.log('Send your Resume!');
            // Remeber Scroll Position
            $(document).ready(function () {
              if (localStorage.getItem('ratemymanagers.xyz-quote-scroll') != null) {
                  $(window).scrollTop(localStorage.getItem('ratemymanagers.xyz-quote-scroll'));
              }
              $(window).on('scroll', function() {
                  localStorage.setItem('ratemymanagers.xyz-quote-scroll', $(window).scrollTop());
              });
            });
          </script>
        "#
        ))
    }
}

fn delete_cookie_js(id: &str, path: &str) -> String {
    format!(
        "document.cookie ='{}=; expires=Thu, 01 Jan 1970 00:00:00 UTC; path={};' ;",
        id, path
    )
}

fn play_audio_without_control(url :&str) -> String {
    format!("(function(){{new Audio('{}').play();}}())", url)
}

fn notification_view(options: &AppModel) -> Markup {
    let id = "notification";
    if let Some(msg) = &options.flash_msg {
        //TODO map proper message to css kind
        let _message_kind: String = msg.split(" ").take(1).collect();
        let msg = msg.split(" ").skip(1).collect::<Vec<&str>>().join(" ");
        html! {
          div class="notification is-warning"
              id=(id)
          {
              button class="delete"
                     onclick={(toggle_js(id,"is-hidden"))(delete_cookie_js("_flash","/"))}
              {}
              (msg)
          }
        }
    } else {
        html! {}
    }
}

fn default_view(model: &AppModel) -> Markup {
    let lang = &model.lang;
    html! {
      head {
          meta charset="utf-8" {}
          meta name="viewport" content="width=device-width, initial-scale=1, maximum-scale=1" {}
          link rel="stylesheet" href="https://cdn.jsdelivr.net/npm/bulma@0.9.1/css/bulma.min.css" {}
          script defer? src="https://use.fontawesome.com/releases/v5.14.0/js/all.js" {}
          script src="https://cdn.jsdelivr.net/npm/jquery@3.5.1/dist/jquery.min.js" {}
          script src="https://cdn.jsdelivr.net/npm/timeago@1.6.7/jquery.timeago.min.js" {}
          title { (TEXT[lang]["site-title"]) }
      }
      body {
        (main_view(model))
        (keypress_js())
        (timeago_js())
        (remember_scroll_js())
        (development_script_tag())
      }
    }
}

fn main_view(model: &AppModel) -> Markup {
    let hidden_inputs = html! {
        input type="hidden" name="user_id" value=(model.user_id) {}
        input type="hidden" name="user_action_type" value=(model.user_action_type) {}
        input type="hidden" name="user_vocab" value=(model.the_word) {}
        input type="hidden" name="user_progress_idx" value=(model.user_progress_idx) {}
        input type="hidden" name="user_vocab_book_idx" value=(model.user_vocab_book_idx) {}
    };

    let circle_icon_with_overlay_z = html! {
        span class="icon" {
            span class="fa-layers fa-fw" {
              i class="fas fa-circle fa-lg has-text-info" {}
              span class="fa-layers-text fa-inverse"
                   style="font-weight:900"
                   { "Z" }
            }
        }
    };
    let circle_icon_with_overlay_x = html! {
        span class="icon" {
            span class="fa-layers fa-fw" {
              i class="fas fa-circle fa-lg has-text-success" {}
              span class="fa-layers-text fa-inverse" data-fa-transform="right-1"
                   style="font-weight:900"
                   { "X" }
            }
        }
    };
    let circle_icon_with_overlay_c = html! {
        span class="icon" {
            span class="fa-layers fa-fw" {
              i class="fas fa-circle fa-lg has-text-danger" {}
              span class="fa-layers-text fa-inverse"
                   style="font-weight:900"
                   { "C" }
            }
        }
    };
    html! {
        section class="hero is-primary is-fullheight " {
            div class="hero-body" {
                div class="container" {
                    div class="columns is-centered" {
                        div class="column is-half-tablet is-one-third-desktop is-one-quarter-widescreen" {
                            div class="box has-text-centered" {
                                p class="title is-1 has-text-black" {(model.the_word)}
                                @if model.user_action_type == "to_answer" {
                                    p class="subtitle is-6" {(model.the_word_type)}
                                    p class="subtitle is-2" {(model.the_word_meaning)}
                                } @else {
                                    p class="subtitle is-6 has-text-grey" {(model.the_word_type)}
                                    p class="subtitle is-2 has-text-black" {(model.the_word_meaning)}
                                }
                            }
                            form action="/iknow" method="post" id="iknow" {
                                (hidden_inputs)
                            }
                            form action="/idontknow" method="post" id="idontknow" {
                                (hidden_inputs)
                            }
                            form action="/iamright" method="post" id="iamright" {
                                (hidden_inputs)
                            }
                            form action="/iamwrong" method="post" id="iamwrong" {
                                (hidden_inputs)
                            }
                            div class="level is-mobile" {
                                div class="level-item" {
                                    button class="button is-black" type="submit" id="Z"
                                           onclick=(play_audio_without_control(uri!(get_prounciation: &model.the_word).to_string().as_ref()))
                                    {
                                        (circle_icon_with_overlay_z)
                                        span { "发音" }
                                    }
                                }
                                @if model.user_action_type == "to_answer" {
                                        div class="level-item" {
                                            button class="button is-black" type="submit" form="iknow" id="X" {
                                                (circle_icon_with_overlay_x)
                                                span { "知道" }
                                            }
                                        }
                                        div class="level-item" {
                                            button class="button is-black" type="submit" form="idontknow" id="C" {
                                                (circle_icon_with_overlay_c)
                                                span { "不知道" }
                                            }
                                        }
                                } @else if model.user_action_type == "to_check" {
                                        div class="level-item" {
                                            button class="button is-black" type="submit" form="iamright" id="X" {
                                                (circle_icon_with_overlay_x)
                                                span { "正确" }
                                            }
                                        }
                                        div class="level-item" {
                                            button class="button is-black" type="submit" form="iamwrong" id="C" {
                                                (circle_icon_with_overlay_c)
                                                span { "记错了" }
                                            }
                                        }
                                } @ else {
                                        div class="level-item" {
                                            button class="button is-black" type="submit" form="iamright" id="X" {
                                                (circle_icon_with_overlay_x)
                                                span { "跳过" }
                                            }
                                        }
                                        div class="level-item" {
                                            button class="button is-black" type="submit" form="iamwrong" id="C" {
                                                (circle_icon_with_overlay_c)
                                                span { "下一个" }
                                            }
                                        }
                               }
                            }
                        }
                    }
                }
            }
        }
    }
}

#[cfg(debug_assertions)]
fn development_script_tag() -> Markup {
    html! {
      script src="http://127.0.0.10:35729/livereload.js" {}
    }
}

#[cfg(not(debug_assertions))]
fn development_script_tag() -> Markup {
    html! {}
}

fn rocket() -> rocket::Rocket {
    rocket::ignite()
        .mount(
            "/",
            routes![
                index,
                favicon,
                robots,
                setlang,
                instantclick,
                hitcount,
                get_prounciation,
                get_next_question_when_right,
                get_next_question_when_wrong,
                check_answer_when_know,
                check_answer_when_dontknow,
            ],
        )
        .manage(HitCount(AtomicUsize::new(0)))
        .attach(db::Connection::fairing())
}

fn main() {
    rocket().launch();
}
