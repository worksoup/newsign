use clap::{Parser, Subcommand};
use std::path::PathBuf;
#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "进行签到。",
    long_about = r#"
进行签到。

关于签到行为：

普通签到不需要指定任何选项。
拍照签到可指定 `-p, --pic` 选项，提供照片位置。如不提供则从云盘上获取图片。
二维码签到可指定 `-p, --pic` 选项，提供照片位置。如不提供则从屏幕上截取。
位置签到可指定 `    --pos` 或 `-l, --location` 选项。如不提供则根据教师设置的签到范围或数据库中获取。
手势或签到码签到须指定 `-s, --signcode` 选项，提供签到码。
"#
)]
pub struct Args {
    #[command(subcommand)]
    pub command: Option<MainCmds>,
    /// 签到 ID.
    /// 默认以最近起对所有有效签到顺序进行签到，且缺少参数时会跳过并继续。
    pub activity: Option<i64>,
    /// 签到账号，格式为以半角逗号隔开的字符串。
    /// 默认以一定顺序对所有用户进行签到。
    #[arg(short, long)]
    pub accounts: Option<String>,
    /// 位置 ID.
    /// 位置签到或二维码位置签到时需要提供。
    /// 也可以通过 `--pos` 选项直接指定位置，此时本选项将失效。
    /// 默认按照先课程位置后全局位置的顺序依次尝试。
    #[arg(short, long)]
    pub location: Option<i64>,
    /// 通过地址名称、经纬度与海拔直接指定位置。
    /// 教师未指定位置签到或二维码签到的签到位置时需要提供。
    /// 格式为：`地址,经度,纬度,海拔`.
    #[arg(long)]
    pub pos: Option<String>,
    /// 本地图片路径。
    /// 拍照签到需要提供，二维码签到可选提供。
    /// 如果是文件，则直接使用该文件作为拍照签到图片或二维码图片文件。
    /// 如果是目录，则会选择在该目录下修改日期最新的图片作为拍照签到图片或二维码图片。
    #[arg(short, long)]
    pub pic: Option<PathBuf>,
    // /// 从屏幕上获取二维码。
    // /// 二维码签到时需要提供。
    // #[arg(short, long)]
    // pub capture: bool,
    /// 精确地截取二维码。
    /// 如果二维码识别过慢可以尝试添加添加此选项。
    #[arg(long)]
    pub precise: bool,
    /// 签到码。
    /// 签到码签到时需要提供。
    #[arg(short, long)]
    pub signcode: Option<String>,
    /// 禁用位置随机偏移。
    #[arg(short, long)]
    pub no_random_shift: bool,
}

#[derive(Subcommand, Debug)]
pub enum MainCmds {
    /// 账号相关操作（列出、添加、删除）。
    /// 默认列出所有账号。
    Account {
        #[command(subcommand)]
        command: Option<AccCmds>,
        /// 重新获取账号信息并缓存。
        #[arg(short, long)]
        fresh: bool,
    },
    /// 列出所有课程。
    Course {
        /// 重新获取课程信息并缓存。
        #[arg(short, long)]
        fresh: bool,
    },
    /// 列出有效签到。
    List {
        /// 列出指定课程的签到。
        #[arg(short, long)]
        course: Option<i64>,
        /// 列出所有签到（包括无效签到）。
        #[arg(short, long)]
        all: bool,
    },
    /// 位置相关操作（列出、添加、删除）。
    /// 默认列出所有位置。
    Pos {
        #[command(subcommand)]
        command: Option<PosCmds>,
        /// 列出绑定指定课程的位置。
        #[arg(short, long)]
        course: Option<i64>,
        /// 列出全局位置。
        #[arg(short, long)]
        global: bool,
    },
    /// 显示配置文件夹位置。
    WhereIsConfig,
}

#[derive(Subcommand, Debug)]
pub enum AccCmds {
    /// 添加账号。
    Add {
        /// 账号（手机号）。
        uname: String,
    },
    /// 删除账号。
    Remove {
        /// 账号（手机号）。
        uname: String,
        /// 无需确认直接删除。
        #[arg(short, long)]
        yes: bool,
    },
}
#[derive(Subcommand, Debug)]
pub enum PosCmds {
    /// 添加位置。
    Add {
        /// 绑定该位置到指定课程。
        /// 默认添加为全局位置。
        #[arg(short, long)]
        course: Option<i64>,
        /// 地址名称、经纬度与海拔。
        /// 格式为：`地址,经度,纬度,海拔`.
        pos: String,
    },
    /// 删除位置。
    Remove {
        /// 位置 ID.
        posid: Option<i64>,
        /// 无需确认直接删除。
        #[arg(short, long)]
        yes: bool,
        #[arg(short, long)]
        all: bool,
    },
    /// 导入位置。
    Import {
        /// 导入位置。
        /// 每行一个位置。课程号在前，位置在后，由字符 `$` 隔开。
        input: PathBuf,
    },
    /// 导入位置。
    Export {
        /// 导出位置。
        /// 无法解析的行将会被跳过。
        output: PathBuf,
    },
}

pub struct CliArgs {
    pub 位置id: Option<i64>,
    pub 位置字符串: Option<String>,
    pub 图片或图片路径: Option<std::path::PathBuf>,
    // pub capture: bool,
    pub 是否精确识别二维码: bool,
    pub 签到码: Option<String>,
    pub 是否禁用随机偏移: bool,
}
