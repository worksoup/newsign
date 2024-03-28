use crate::utils::pic_dir_or_path_to_pic_path;
use crate::SignnerTrait;
use cxsign_activity::sign::{QrCodeSign, SignResult, SignTrait};
use cxsign_error::Error;
use cxsign_store::{DataBase, DataBaseTableTrait};
use cxsign_types::{Location, LocationTable, LocationWithRange};
use cxsign_user::Session;
use log::warn;
use std::collections::HashMap;
use std::path::PathBuf;

pub struct DefaultQrCodeSignner<'a> {
    db: &'a DataBase,
    location_str: &'a Option<String>,
    path: &'a Option<PathBuf>,
    enc: &'a Option<String>,
    precisely: bool,
    no_rand_shift: bool,
}

impl<'a> DefaultQrCodeSignner<'a> {
    pub fn new(
        db: &'a DataBase,
        location_str: &'a Option<String>,
        path: &'a Option<PathBuf>,
        enc: &'a Option<String>,
        precisely: bool,
        no_rand_shift: bool,
    ) -> Self {
        Self {
            db,
            location_str,
            path,
            enc,
            precisely,
            no_rand_shift,
        }
    }
}
impl SignnerTrait<QrCodeSign> for DefaultQrCodeSignner<'_> {
    fn sign<'a, Sessions: Iterator<Item = &'a Session> + Clone>(
        &self,
        sign: &mut QrCodeSign,
        sessions: Sessions,
    ) -> Result<HashMap<&'a Session, SignResult>, Error> {
        let locations =
            match crate::utils::通过位置字符串决定位置(self.db, self.location_str) {
                Ok(位置) => {
                    vec![位置]
                }
                Err(位置字符串) => {
                    let mut 预设位置列表 = HashMap::new();
                    for session in sessions.clone() {
                        预设位置列表 =
                            LocationWithRange::from_log(session, &sign.as_inner().course)?;
                        break;
                    }
                    let 预设位置 = 预设位置列表.get(&sign.as_inner().active_id).map(|l| {
                        if self.no_rand_shift {
                            l.to_location()
                        } else {
                            l.to_shifted_location()
                        }
                    });
                    let table = LocationTable::from_ref(self.db);
                    let locations = if 位置字符串.is_empty() {
                        let mut 全局位置列表 = table.get_location_list_by_course(-1);
                        let mut 位置列表 =
                            table.get_location_list_by_course(sign.as_inner().course.get_id());
                        全局位置列表.append(&mut 位置列表);
                        if let Some(location) = 预设位置 {
                            全局位置列表.push(location)
                        }
                        全局位置列表
                    } else {
                        let 预设位置 = 预设位置.map(|l| {
                            Location::new(&位置字符串, l.get_lon(), l.get_lat(), l.get_alt())
                        });
                        if let Some(location) = 预设位置 {
                            vec![location]
                        } else {
                            vec![]
                        }
                    };
                    locations
                }
            };
        let enc = if let Some(enc) = self.enc {
            enc.clone()
        } else if let Some(pic) = self.path {
            if std::fs::metadata(pic).unwrap().is_dir() {
                if let Some(pic) = pic_dir_or_path_to_pic_path(pic)?
                    && let Some(enc) =
                        crate::utils::扫描路径中二维码并获取签到所需参数(
                            pic.to_str().unwrap(),
                        )
                {
                    enc
                } else {
                    return Err(Error::EncError(
                        "图片文件夹下没有图片（`png` 或 `jpg` 文件）！".to_owned(),
                    ));
                }
            } else if let Some(enc) =
                crate::utils::扫描路径中二维码并获取签到所需参数(
                    pic.to_str().unwrap(),
                )
            {
                enc
            } else {
                return Err(Error::EncError("二维码中没有 `enc` 参数！".to_owned()));
            }
        } else if let Some(enc) = crate::utils::截屏获取二维码签到所需参数(
            match sign {
                QrCodeSign::RefreshQrCodeSign(_) => true,
                QrCodeSign::NormalQrCodeSign(_) => false,
            },
            self.precisely,
        ) {
            enc
        } else {
            return Err(Error::EncError("截屏时未获取到 `enc` 参数！".to_owned()));
        };
        sign.set_enc(enc);
        let mut map = HashMap::new();
        for session in sessions {
            let state = match sign.pre_sign(session)? {
                SignResult::Susses => SignResult::Susses,
                SignResult::Fail { .. } => {
                    let mut state = SignResult::Fail {
                        msg: "所有位置均不可用".into(),
                    };
                    for location in &locations {
                        sign.set_location(location.clone());
                        match self.sign_single(sign, session)? {
                            r @ SignResult::Susses => {
                                state = r;
                                break;
                            }
                            SignResult::Fail { msg } => {
                                if msg == "您已签到过了".to_owned() {
                                    state = SignResult::Susses;
                                    break;
                                } else {
                                    warn!(
                                        "用户[{}]在二维码签到[{}]中尝试位置[{}]时失败！失败信息：[{:?}]",
                                        session.get_stu_name(),
                                        sign.as_inner().name,
                                        location,msg
                                    );
                                }
                            }
                        };
                    }
                    state
                }
            };
            map.insert(session, state);
        }
        Ok(map)
    }

    fn sign_single(&self, sign: &mut QrCodeSign, session: &Session) -> Result<SignResult, Error> {
        unsafe { sign.sign_unchecked(session).map_err(|e| e.into()) }
    }
}
