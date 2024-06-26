use ureq::{Agent, Response};

// 账号设置页
static ACCOUNT_MANAGE: &str = "https://passport2.chaoxing.com/mooc/accountManage";

pub fn account_manage(client: &Agent) -> Result<Response, Box<ureq::Error>> {
    Ok(client.get(ACCOUNT_MANAGE).call()?)
}
