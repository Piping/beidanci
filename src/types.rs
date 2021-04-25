use std::{fmt, str::FromStr};

use rocket::{
    http::RawStr,
    request,
    request::{FromParam, FromRequest, Request},
    Outcome,
};

use strum::{EnumIter, EnumString, IntoStaticStr};

#[derive(
    Debug, Copy, Clone, PartialEq, Eq, Hash, IntoStaticStr, EnumString, EnumIter, strum::ToString,
)]
pub enum PanelRankType {
    #[strum(serialize = "Most Recent")]
    MostRecent,
    #[strum(serialize = "Most Reviewed")]
    MostReview,
    #[strum(serialize = "Most Liked")]
    MostLike,
}

impl From<&str> for PanelRankType {
    fn from(s: &str) -> Self {
        PanelRankType::from_str(s).unwrap()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum ServerAcceptLangauge {
    SimpliedChinese,
    Japananese,
    English,
}
impl Default for ServerAcceptLangauge {
    fn default() -> Self {
        ServerAcceptLangauge::English
    }
}
impl<'a, 'r> FromRequest<'a, 'r> for ServerAcceptLangauge {
    type Error = &'r RawStr;

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let first_lang: Option<&str> = request.headers().get("accept-language").next();
        match first_lang {
            // TODO process raw string here
            Some(lang) => {
                if lang.contains("zh") {
                    Outcome::Success(ServerAcceptLangauge::SimpliedChinese)
                } else if lang.contains("jp") {
                    Outcome::Success(ServerAcceptLangauge::Japananese)
                } else {
                    Outcome::Success(ServerAcceptLangauge::English)
                }
            }
            None => Outcome::Success(ServerAcceptLangauge::English),
        }
    }
}
impl<'r> FromParam<'r> for ServerAcceptLangauge {
    type Error = &'r RawStr;

    fn from_param(param: &'r RawStr) -> Result<Self, Self::Error> {
        Ok(ServerAcceptLangauge::from(param.as_str()))
    }
}

impl From<&str> for ServerAcceptLangauge {
    fn from(s: &str) -> Self {
        match s {
            "zh" => (ServerAcceptLangauge::SimpliedChinese),
            "jp" => (ServerAcceptLangauge::Japananese),
            "en" => (ServerAcceptLangauge::English),
            _ => (ServerAcceptLangauge::English),
        }
    }
}

impl fmt::Display for ServerAcceptLangauge {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ServerAcceptLangauge::SimpliedChinese => write!(f, "zh"),
            ServerAcceptLangauge::Japananese => write!(f, "jp"),
            ServerAcceptLangauge::English => write!(f, "en"),
        }
    }
}
