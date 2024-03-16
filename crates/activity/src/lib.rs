#![feature(let_chains)]

pub mod protocol;
pub mod sign;

use crate::sign::{RawSign, Sign, SignTrait};
use serde::{Deserialize, Serialize};
use types::Course;
use user::session::Session;

#[derive(Debug)]
pub enum Activity {
    Sign(Sign),
    Other(OtherActivity),
}

impl Activity {
    pub fn get_all_activities(
        session: &Session,
    ) -> Result<(Vec<Sign>, Vec<Sign>, Vec<OtherActivity>), ureq::Error> {
        let mut 有效签到列表 = Vec::new();
        let mut 其他签到列表 = Vec::new();
        let mut 非签到活动列表 = Vec::new();
        let 课程列表 = Course::get_courses(session)?;
        for c in 课程列表 {
            let item = Self::get_list_from_course(session, &c)?;
            for a in item {
                if let Self::Sign(签到) = a {
                    if 签到.is_valid() {
                        有效签到列表.push(签到);
                    } else {
                        其他签到列表.push(签到);
                    }
                } else if let Self::Other(非签到活动) = a {
                    非签到活动列表.push(非签到活动);
                }
            }
        }
        有效签到列表.sort();
        Ok((有效签到列表, 其他签到列表, 非签到活动列表))
    }
    pub fn get_list_from_course(session: &Session, c: &Course) -> Result<Vec<Self>, ureq::Error> {
        let r = crate::protocol::active_list(session, c.clone())?;
        let r: GetActivityR = r.into_json().unwrap();
        let mut activities = Vec::new();
        if let Some(data) = r.data {
            for ar in data.active_list {
                if let Some(other_id) = ar.other_id
                    && {
                        let other_id_i64: i64 = other_id.parse().unwrap();
                        (0..=5).contains(&other_id_i64)
                    }
                {
                    let active_id = ar.id.to_string();
                    let detail = RawSign::get_sign_detail(active_id.as_str(), session)?;
                    let base_sign = RawSign {
                        active_id,
                        name: ar.name_one,
                        course: c.clone(),
                        other_id,
                        status_code: ar.status,
                        start_timestamp: (ar.start_time / 1000) as i64,
                        sign_detail: detail,
                    };
                    activities.push(Self::Sign(base_sign.to_sign()))
                } else {
                    activities.push(Self::Other(OtherActivity {
                        id: ar.id.to_string(),
                        name: ar.name_one,
                        course: c.clone(),
                        status: ar.status,
                        start_time_secs: (ar.start_time / 1000) as i64,
                    }))
                }
            }
        }
        Ok(activities)
    }
}

#[derive(Debug)]
pub struct OtherActivity {
    pub id: String,
    pub name: String,
    pub course: Course,
    pub status: i32,
    pub start_time_secs: i64,
}

#[derive(Deserialize, Serialize)]
struct ActivityRaw {
    #[serde(alias = "nameOne")]
    name_one: String,
    id: i64,
    #[serde(alias = "otherId")]
    other_id: Option<String>,
    status: i32,
    #[serde(alias = "startTime")]
    start_time: u64,
}

#[derive(Deserialize, Serialize)]
struct Data {
    #[serde(alias = "activeList")]
    active_list: Vec<ActivityRaw>,
}

#[derive(Deserialize, Serialize)]
struct GetActivityR {
    data: Option<Data>,
}
