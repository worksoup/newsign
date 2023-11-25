mod single_sign;

use futures::{stream::FuturesUnordered, StreamExt};
use tokio::sync::{Mutex, RwLock};

use crate::{
    sign_session::{
        activity::sign::{SignActivity, SignState, SignType},
        session::SignSession,
    },
    utils::{address::Address, handle_qrcode_pic_path, photo::Photo, picdir_to_pic, sql::DataBase},
};
use std::{collections::HashMap, path::PathBuf, sync::Arc};

async fn general_sign_<'a>(
    sign: &SignActivity,
    sessions: &'a Vec<&SignSession>,
) -> Result<HashMap<&'a str, SignState>, reqwest::Error> {
    let mut states = HashMap::new();
    let mut tasks = FuturesUnordered::new();
    for session in sessions {
        tasks.push(single_sign::general_sign_single(sign, session));
    }
    while let Some(tmp) = tasks.next().await {
        let (name, state) = tmp?;
        states.insert(name, state);
    }
    Ok(states)
}

async fn photo_sign_<'a>(
    sign: &SignActivity,
    pic: &Option<PathBuf>,
    sessions: &'a Vec<&SignSession>,
) -> Result<HashMap<&'a str, SignState>, reqwest::Error> {
    let mut states = HashMap::new();
    let photo = if let Some(pic) = &pic {
        Photo::from_file(sessions[0], &pic).await
    } else {
        Photo::default(sessions[0]).await
    };
    let mut tasks = FuturesUnordered::new();
    for session in sessions {
        tasks.push(single_sign::photo_sign_single(sign, &photo, session));
    }
    while let Some(tmp) = tasks.next().await {
        let (name, state) = tmp?;
        states.insert(name, state);
    }
    Ok(states)
}
async fn qrcode_sign_<'a>(
    sign: &SignActivity,
    c: &str,
    enc: &str,
    poss: &Vec<Address>,
    sessions: &'a Vec<&SignSession>,
) -> Result<HashMap<&'a str, SignState>, reqwest::Error> {
    let mut states = HashMap::new();
    let mut tasks = FuturesUnordered::new();
    for session in sessions {
        tasks.push(single_sign::qrcode_sign_single(sign, c, enc, poss, session));
    }
    while let Some(tmp) = tasks.next().await {
        let (name, state) = tmp?;
        states.insert(name, state);
    }
    Ok(states)
}

async fn location_sign_<'a>(
    sign: &SignActivity,
    poss: &Vec<Address>,
    sessions: &'a Vec<&SignSession>,
) -> Result<HashMap<&'a str, SignState>, reqwest::Error> {
    let mut states = HashMap::new();
    let mut tasks = FuturesUnordered::new();
    for session in sessions {
        tasks.push(single_sign::location_sign_single(sign, poss, session));
    }
    while let Some(tmp) = tasks.next().await {
        let (name, state) = tmp?;
        states.insert(name, state);
    }
    Ok(states)
}

async fn signcode_sign_<'a>(
    sign: &SignActivity,
    signcode: &str,
    sessions: &'a Vec<&SignSession>,
) -> Result<HashMap<&'a str, SignState>, reqwest::Error> {
    let mut states = HashMap::new();
    let mut tasks = FuturesUnordered::new();
    for session in sessions {
        tasks.push(single_sign::signcode_sign_single(sign, signcode, session));
    }
    while let Some(tmp) = tasks.next().await {
        let (name, state) = tmp?;
        states.insert(name, state);
    }
    Ok(states)
}
async fn location_and_pos_to_poss(
    sign: &SignActivity,
    db: &DataBase,
    location: Option<i64>,
    pos: &Option<String>,
) -> Vec<Address> {
    if let Some(ref pos) = pos {
        vec![Address::parse_str(pos)]
    } else if let Some(addr) = location {
        let poss = db.get_pos(addr);
        vec![poss.1]
    } else {
        let mut poss = db.get_course_poss_without_posid(sign.course.get_id());
        let mut other = db.get_course_poss_without_posid(-1);
        poss.append(&mut other);
        poss
    }
}
async fn handle_account_sign<'a>(
    sign: &SignActivity,
    pic: &Option<PathBuf>,
    location: Option<i64>,
    db: &DataBase,
    pos: &Option<String>,
    signcode: &Option<String>,
    sessions: &'a Vec<&SignSession>,
) -> Result<(), reqwest::Error> {
    let sign_type = sign.get_sign_type();
    let mut states = HashMap::new();

    match sign_type {
        SignType::Photo => {
            if let Some(pic) = pic {
                if let Ok(metadata) = std::fs::metadata(&pic) {
                    let pic = if metadata.is_dir() {
                        crate::utils::picdir_to_pic(&pic)
                    } else {
                        Some(pic.to_owned())
                    };
                    states = photo_sign_(sign, &pic, sessions).await?;
                } else {
                    eprintln!(
                        "所有用户在拍照签到[{}]中签到失败！未能获取{:?}的元信息！",
                        sign.name, pic
                    );
                };
            } else {
                eprintln!(
                    "所有用户在拍照签到[{}]中签到失败！未提供照片路径！",
                    sign.name
                )
            };
        }
        SignType::Common => {
            states = general_sign_(sign, sessions).await?;
        }
        SignType::QrCode => {
            let poss = location_and_pos_to_poss(sign, db, location, pos).await;
            if let Some(pic) = pic {
                let metadata = std::fs::metadata(&pic).unwrap();
                if metadata.is_dir() {
                    if let Some(pic) = picdir_to_pic(&pic) {
                        let enc = handle_qrcode_pic_path(pic.to_str().unwrap());
                        states =
                            qrcode_sign_(sign, sign.get_c_of_qrcode_sign(), &enc, &poss, sessions)
                                .await?;
                    } else {
                        eprintln!(
                            "所有用户在二维码签到[{}]中签到失败！二维码签到需要提供或签到二维码！",
                            sign.name
                        );
                    }
                } else {
                    let enc = handle_qrcode_pic_path(pic.to_str().unwrap());
                    states = qrcode_sign_(sign, sign.get_c_of_qrcode_sign(), &enc, &poss, sessions)
                        .await?;
                }
            } else {
                eprintln!(
                    "所有用户在二维码签到[{}]中签到失败！二维码签到需要提供签到二维码！",
                    sign.name
                );
            };
        }
        SignType::Location => {
            let poss = location_and_pos_to_poss(sign, db, location, pos).await;
            states = location_sign_(sign, &poss, sessions).await?;
        }
        SignType::Unknown => {
            eprintln!("签到活动[{}]为无效签到类型！", sign.name);
        }
        signcode_sign_type => {
            if let Some(signcode) = signcode {
                states = signcode_sign_(sign, signcode, sessions).await?;
            } else {
                let sign_type_str = match signcode_sign_type {
                    SignType::Gesture => "手势",
                    SignType::SignCode => "签到码",
                    _ => unreachable!(),
                };
                eprintln!(
                    "所有用户在{sign_type_str}签到[{}]中签到失败！需要提供签到码！",
                    sign.name
                )
            }
        }
    };
    if !states.is_empty() {
        println!("签到活动[{}]签到结果：", sign.name);
        for (uname, state) in states {
            if let SignState::Fail(msg) = state {
                eprintln!("\t用户[{}]签到失败！失败信息：[{:?}]", uname, msg);
            } else {
                println!("\t用户[{}]签到成功！", uname,);
            }
        }
    }
    Ok(())
}

pub async fn sign(
    db: &DataBase,
    sessions: &HashMap<String, SignSession>,
    asigns: HashMap<SignActivity, Vec<&SignSession>>,
    osigns: HashMap<SignActivity, Vec<&SignSession>>,
    activity: Option<i64>,
    account: Option<String>,
    location: Option<i64>,
    pos: Option<String>,
    pic: Option<PathBuf>,
    signcode: Option<String>,
) -> Result<(), reqwest::Error> {
    if let Some(active_id) = activity {
        let s1 = asigns.iter().find(|kv| kv.0.id == active_id.to_string());
        let s2 = osigns.iter().find(|kv| kv.0.id == active_id.to_string());
        let (sign, mut full_sessions) = {
            if let Some(s1) = s1 {
                s1
            } else if let Some(s2) = s2 {
                s2
            } else {
                panic!("没有该签到活动！请检查签到活动 ID 是否正确！");
            }
        };
        let mut accounts = Vec::new();
        if let Some(ref account) = &account {
            accounts.push(&sessions[account]);
            full_sessions = &accounts;
        }
        handle_account_sign(sign, &pic, location, db, &pos, &signcode, full_sessions).await?;
    } else {
        for (sign, mut full_sessions) in &asigns {
            let mut accounts = Vec::new();
            if let Some(ref account) = &account {
                accounts.push(&sessions[account]);
                full_sessions = &accounts;
            }
            handle_account_sign(sign, &pic, location, db, &pos, &signcode, full_sessions).await?;
        }
    }
    Ok(())
}